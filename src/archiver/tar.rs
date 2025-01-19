use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use flate2::write::GzEncoder;
use bzip2::write::BzEncoder;
use tar::Builder;
use xz2::write::XzEncoder;

use crate::archiver::{ToteArchiver, Format, ArchiverOpts};
use crate::{ToteError, Result};

use super::TargetPath;

pub(super) struct TarArchiver {
}
pub(super) struct TarGzArchiver {
}
pub(super) struct TarBz2Archiver {
}
pub(super) struct TarXzArchiver {
}
pub(super) struct TarZstdArchiver {
}

impl ToteArchiver for  TarArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        write_tar(tps, file)
    }
    fn format(&self) -> Format {
        Format::Tar
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarGzArchiver{
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        write_tar(tps, GzEncoder::new(file, flate2::Compression::default()))
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarBz2Archiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        write_tar(tps, BzEncoder::new(file, bzip2::Compression::best()))
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarXzArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        write_tar(tps, XzEncoder::new(file, 9))
    }

    fn format(&self) -> Format {
        Format::TarXz
    }
    fn enable(&self) -> bool {
        true
    }
}

impl ToteArchiver for TarZstdArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _: &ArchiverOpts) -> Result<()> {
        let encoder = zstd::Encoder::new(file, 9).unwrap();
        write_tar(tps, encoder.auto_finish())
    }

    fn format(&self) -> Format {
        Format::TarZstd
    }
    fn enable(&self) -> bool {
        true
    }
}

fn write_tar<W: Write>(tps: Vec<TargetPath>, f: W) -> Result<()> {
    let mut builder = tar::Builder::new(f);
    let mut errs = vec![];
    for tp in tps {
        for entry in tp.walker() {
            if let Ok(t) = entry {
                let path = t.into_path();
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
    }
    if let Err(e) = builder.finish() {
        errs.push(ToteError::Archiver(e.to_string()));
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(ToteError::Array(errs))
    }
}

fn process_file<W: Write>(builder: &mut Builder<W>, target: &PathBuf, dest_path: &PathBuf) -> Result<()> {
    if let Err(e) = builder.append_path_with_name(target, dest_path) {
        Err(ToteError::Archiver(e.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::archiver::Archiver;
    use crate::archiver::ArchiverOpts;
    use crate::format::Format;

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
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(
                PathBuf::from("results/test.tar"), 
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], 
                opts).unwrap();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar");
            if let Err(e) = result {
                panic!("{:?}", e);
            }
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::Tar);
            path
        });
    }

    
    #[test]
    fn test_targz() {
        run_test(|| {
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(
                PathBuf::from("results/test.tar.gz"), 
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], 
                opts).unwrap();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.gz");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::TarGz);
            path
        });
    }

    #[test]
    fn test_tarbz2() {
        run_test(|| {
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(
                PathBuf::from("results/test.tar.bz2"), 
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], 
                opts).unwrap();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.bz2");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::TarBz2);
            path
        });
    }

    #[test]
    fn test_tarxz() {
        run_test(|| {
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(
                PathBuf::from("results/test.tar.xz"), 
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], 
                opts).unwrap();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.xz");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::TarXz);
            path
        });
    }

    #[test]
    fn test_tarzstd() {
        run_test(|| {
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(
                PathBuf::from("results/test.tar.zst"), 
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], 
                opts).unwrap();
            let result = archiver.perform();
            let path = PathBuf::from("results/test.tar.zst");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::TarZstd);
            path
        });
    }

    fn teardown(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }
}
