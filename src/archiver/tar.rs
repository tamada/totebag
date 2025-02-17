use bzip2::write::BzEncoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tar::Builder;
use xz2::write::XzEncoder;

use crate::archiver::{ArchiveEntry, Targets, ToteArchiver};
use crate::{Result, ToteError};

pub(super) struct TarArchiver {}
pub(super) struct TarGzArchiver {}
pub(super) struct TarBz2Archiver {}
pub(super) struct TarXzArchiver {}
pub(super) struct TarZstdArchiver {}

impl ToteArchiver for TarArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        write_tar(tps, file)
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarGzArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let level = tps.level() as u32;
        write_tar(tps, GzEncoder::new(file, flate2::Compression::new(level)))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarBz2Archiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let level = tps.level() as u32;
        write_tar(tps, BzEncoder::new(file, bzip2::Compression::new(level)))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarXzArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let level = tps.level() as u32;
        write_tar(tps, XzEncoder::new(file, level))
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarZstdArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let level = tps.level() as u32;
        let level = (level as f64 + 1.0) / 10.0 * 22.0; // convert to 1-22
        let encoder = zstd::Encoder::new(file, level as i32).unwrap();
        write_tar(tps, encoder.auto_finish())
    }
    fn enable(&self) -> bool {
        true
    }
}

fn write_tar<W: Write>(tps: Targets, f: W) -> Result<Vec<ArchiveEntry>> {
    let mut builder = tar::Builder::new(f);
    let mut errs = vec![];
    let mut entries = vec![];
    for tp in tps.iter() {
        for t in tp.iter() {
            let path = t.into_path();
            entries.push(ArchiveEntry::from(&path));
            let dest_dir = tp.dest_path(&path);
            if path.is_file() {
                if let Err(e) = process_file(&mut builder, &path, &dest_dir) {
                    errs.push(e);
                }
            } else if path.is_dir() {
                if let Err(e) = builder.append_dir(&dest_dir, &path) {
                    errs.push(ToteError::Archiver(e.to_string()));
                }
            }
        }
    }
    if let Err(e) = builder.finish() {
        errs.push(ToteError::Archiver(e.to_string()));
    }
    if errs.is_empty() {
        Ok(entries)
    } else {
        Err(ToteError::Array(errs))
    }
}

fn process_file<W: Write>(
    builder: &mut Builder<W>,
    target: &PathBuf,
    dest_path: &PathBuf,
) -> Result<()> {
    if let Err(e) = builder.append_path_with_name(target, dest_path) {
        Err(ToteError::Archiver(e.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::Archiver;
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
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.tar"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
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
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.tar.gz"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.gz");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarbz2() {
        run_test(|| {
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.tar.bz2"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.bz2");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarxz() {
        run_test(|| {
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.tar.xz"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.xz");
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    #[test]
    fn test_tarzstd() {
        run_test(|| {
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.tar.zst"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
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
