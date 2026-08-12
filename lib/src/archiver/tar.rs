use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use lzma_rust2::{XzOptions, XzWriter};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tar::Builder;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Error, Result};

/// TAR format archiver implementation.
pub(super) struct Archiver {}

/// TAR+GZIP format archiver implementation.
pub(super) struct GzArchiver {}

/// TAR+BZIP2 format archiver implementation.
pub(super) struct Bz2Archiver {}

/// TAR+XZ format archiver implementation.
pub(super) struct XzArchiver {}

/// TAR+ZSTD format archiver implementation.
pub(super) struct ZstdArchiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        write_tar(file, targets, config)
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for GzArchiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        write_tar(
            GzEncoder::new(file, flate2::Compression::new(level)),
            targets,
            config,
        )
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for Bz2Archiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        write_tar(
            BzEncoder::new(file, bzip2::Compression::new(level)),
            targets,
            config,
        )
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for XzArchiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        let encoder = XzWriter::new(file, XzOptions::with_preset(level))
            .map_err(|e| Error::Archiver(e.to_string()))?;
        write_tar(encoder.auto_finish(), targets, config)
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for ZstdArchiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        zstd::write_tar_zstd(file, targets, config)
    }

    fn enable(&self) -> bool {
        true
    }
}

/// The zstd encoder backend.
///
/// The pure Rust `ruzstd` compressor pulls from a reader instead of exposing a
/// [`Write`] sink, so the tar stream is staged in a temporary file and compressed
/// afterwards. Enabling the `zstd-native` feature swaps in the C library, which
/// streams directly and honours the whole 0-9 level range.
#[cfg(not(feature = "zstd-native"))]
mod zstd {
    use super::{ArchiveEntry, PathBuf, write_tar};
    use crate::{Error, Result};
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    pub(super) fn write_tar_zstd(
        mut file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut staging = tempfile::tempfile().map_err(Error::IO)?;
        let entries = write_tar(&mut staging, targets, config)?;
        staging.seek(SeekFrom::Start(0)).map_err(Error::IO)?;

        // `ruzstd` implements only `Fastest`; the other levels are placeholders
        // in the crate. `level == 0` still produces a valid zstd frame.
        let level = if config.level == 0 {
            ruzstd::encoding::CompressionLevel::Uncompressed
        } else {
            ruzstd::encoding::CompressionLevel::Fastest
        };
        let mut compressor = ruzstd::encoding::FrameCompressor::new(level);
        compressor.set_source(&mut staging);
        compressor.set_drain(&mut file);
        compressor.compress();
        Ok(entries)
    }
}

#[cfg(feature = "zstd-native")]
mod zstd {
    use super::{ArchiveEntry, PathBuf, write_tar};
    use crate::{Error, Result};
    use std::fs::File;

    pub(super) fn write_tar_zstd(
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = level_to_zstd(config.level);
        let encoder =
            ::zstd::Encoder::new(file, level).map_err(|e| Error::Archiver(e.to_string()))?;
        write_tar(encoder.auto_finish(), targets, config)
    }

    /// Maps the totebag level (0-9) onto the zstd level range (1-22).
    fn level_to_zstd(level: u8) -> i32 {
        match level {
            0 => 1,
            9 => 22,
            l => (f64::from(l) / 9.0 * 21.0).round() as i32 + 1,
        }
    }
}

fn write_tar<W: Write>(
    f: W,
    targets: &[PathBuf],
    config: &crate::ArchiveConfig,
) -> Result<Vec<ArchiveEntry>> {
    let mut builder = tar::Builder::new(f);
    let mut errs = vec![];
    let mut entries = vec![];
    for tp in targets {
        for entry in config.iter(tp) {
            let path = entry.into_path();
            let dest_dir = config.path_in_archive(&path);
            // A directory such as `..` normalizes away to nothing; there is no
            // name to store it under, and its contents are archived anyway.
            if dest_dir.as_os_str().is_empty() {
                continue;
            }
            entries.push(ArchiveEntry::from(&path));
            if path.is_file() {
                if let Err(e) = process_file(&mut builder, &path, &dest_dir) {
                    errs.push(e);
                }
            } else if path.is_dir()
                && let Err(e) = builder.append_dir(&dest_dir, &path)
            {
                errs.push(Error::Archiver(e.to_string()));
            }
        }
    }
    if let Err(e) = builder.finish() {
        errs.push(Error::Archiver(e.to_string()));
    }
    Error::error_or(entries, errs)
}

fn process_file<W: Write>(
    builder: &mut Builder<W>,
    target: &PathBuf,
    dest_path: &PathBuf,
) -> Result<()> {
    if let Err(e) = builder.append_path_with_name(target, dest_path) {
        Err(Error::Archiver(e.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::test_support::targets;
    use std::path::PathBuf;

    fn run_test<F>(f: F)
    where
        F: FnOnce() -> PathBuf,
    {
        // setup(); // preprocessing process
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match result {
            Ok(path) => teardown(path),
            Err(err) => std::panic::resume_unwind(err),
        }
    }

    #[test]
    fn test_tar() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.tar")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.tar");
            if let Err(e) = result {
                panic!("{:?}", e);
            }
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_targz() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.tar.gz")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.tar.gz");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarbz2() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.tar.bz2")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.tar.bz2");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarxz() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.tar.xz")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.tar.xz");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarzstd() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.tar.zst")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.tar.zst");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    fn teardown(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }
}
