use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tar::Builder;
use xz2::write::XzEncoder;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, Error};

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
        write_tar(XzEncoder::new(file, level), targets, config)
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
        let level = config.level as u32;
        let level = (level as f64 + 1.0) / 10.0 * 22.0; // convert to 1-22
        let encoder = zstd::Encoder::new(file, level as i32).unwrap();
        write_tar(encoder.auto_finish(), targets, config)
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
            entries.push(ArchiveEntry::from(&path));
            let dest_dir = config.path_in_archive(&path);
            if path.is_file() {
                if let Err(e) = process_file(&mut builder, &path, &dest_dir) {
                    errs.push(e);
                }
            } else if path.is_dir() {
                if let Err(e) = builder.append_dir(&dest_dir, &path) {
                    errs.push(Error::Archiver(e.to_string()));
                }
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
