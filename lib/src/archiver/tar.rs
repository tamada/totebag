use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use lzma_rust2::{XzOptions, XzWriter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use structured_zstd::encoding::{CompressionLevel, StreamingEncoder};
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
        file: &mut BufWriter<File>,
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
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        let mut encoder = GzEncoder::new(file, flate2::Compression::new(level));
        let r = write_tar(&mut encoder, targets, config);
        let f = encoder.finish().map_err(Error::IO);
        r.and_then(|entries| f.map(|_| entries))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for Bz2Archiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        let mut encoder = BzEncoder::new(file, bzip2::Compression::new(level));
        let r = write_tar(&mut encoder, targets, config);
        let f = encoder.finish().map_err(Error::IO);
        r.and_then(|entries| f.map(|_| entries))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for XzArchiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = config.level as u32;
        let mut encoder = XzWriter::new(file, XzOptions::with_preset(level))
            .map_err(|e| Error::Archiver(e.to_string()))?;
        let r = write_tar(&mut encoder, targets, config);
        let f = encoder.finish().map_err(Error::IO);
        r.and_then(|entries| f.map(|_| entries))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ZstdArchiver {
    /// Maps the totebag level (0-9) onto zstd: 0 stores without compression,
    /// 1-9 map linearly onto zstd's 3-22. Values above 9 are treated as 9.
    fn level_to_zstd(level: u8) -> CompressionLevel {
        match level.min(9) {
            0 => CompressionLevel::Uncompressed,
            9 => CompressionLevel::Level(22),
            l => {
                let l = (f64::from(l) / 9.0 * 21.0).round() as i32 + 1;
                CompressionLevel::Level(l)
            }
        }
    }
}

impl ToteArchiver for ZstdArchiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = Self::level_to_zstd(config.level);
        let mut encoder = StreamingEncoder::new(file, level);
        let r = write_tar(&mut encoder, targets, config);
        let f = encoder.finish().map_err(|e| Error::Archiver(e.to_string()));
        r.and_then(|entries| f.map(|_| entries))
    }

    fn enable(&self) -> bool {
        true
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
            let dest_path = config.path_in_archive(&path);
            // A directory such as `..` normalizes away to nothing; there is no
            // name to store it under, and its contents are archived anyway.
            if dest_path.as_os_str().is_empty() {
                continue;
            }
            entries.push(ArchiveEntry::from(&path));
            if path.is_file() {
                if let Err(e) = process_file(&mut builder, &path, &dest_path) {
                    errs.push(e);
                }
            } else if path.is_dir()
                && let Err(e) = builder.append_dir(&dest_path, &path)
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
    use structured_zstd::encoding::CompressionLevel;

    use crate::archiver::{tar::ZstdArchiver, test_support::targets};
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
    fn test_level_to_zstd() {
        assert!(matches!(
            ZstdArchiver::level_to_zstd(0),
            CompressionLevel::Uncompressed
        ));
        assert!(matches!(
            ZstdArchiver::level_to_zstd(9),
            CompressionLevel::Level(22)
        ));
        assert!(matches!(
            ZstdArchiver::level_to_zstd(200),
            CompressionLevel::Level(22)
        ));
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
