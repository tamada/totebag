use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use flate2::write::GzEncoder;
use bzip2::write::BzEncoder;
use tar::Builder;
use xz2::write::XzEncoder;

use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::cli::{ToteError, Result};

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

impl Archiver for  TarArchiver {
    fn perform(&self, inout: &ArchiverOpts) -> Result<()> {
        write_tar(inout, |file| file)
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}

impl Archiver for TarGzArchiver{
    fn perform(&self, inout: &ArchiverOpts) -> Result<()> {
        write_tar(inout, |file| GzEncoder::new(file, flate2::Compression::default()))
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}

impl Archiver for  TarBz2Archiver {
    fn perform(&self, opts: &ArchiverOpts) -> Result<()> {
        write_tar(opts, |file| BzEncoder::new(file, bzip2::Compression::best()))
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}

impl Archiver for  TarXzArchiver {
    fn perform(&self, inout: &ArchiverOpts) -> Result<()> {
        write_tar(inout, |file| XzEncoder::new(file, 9))
    }

    fn format(&self) -> Format {
        Format::TarXz
    }
}

impl Archiver for  TarZstdArchiver {
    fn perform(&self, inout: &ArchiverOpts) -> Result<()> {
        write_tar(inout, |file| 
            zstd::Encoder::new(file, 9).unwrap())
    }

    fn format(&self) -> Format {
        Format::TarZstd
    }
}

fn write_tar<F, W: Write>(opts: &ArchiverOpts, f: F) -> Result<()> 
        where F: FnOnce(File) -> W {
    match opts.destination() {
        Err(e) => Err(e),
        Ok(file) => {
            let enc = f(file);
            write_tar_impl(enc, opts.targets(), opts.recursive)
        }
    }
}

fn write_tar_impl<W: Write>(file: W, targets: Vec<PathBuf>, recursive: bool) -> Result<()> {
    let mut builder = tar::Builder::new(file);
    for target in targets {
        let path = target.as_path();
        if path.is_dir() && recursive {
            process_dir(&mut builder, path.to_path_buf(), recursive)?
        } else {
            process_file(&mut builder, path.to_path_buf())?
        }
    }
    if let Err(e) = builder.finish() {
        return Err(ToteError::Archiver(e.to_string()))
    }
    Ok(())
}

fn process_dir<W: Write>(builder: &mut Builder<W>, target: PathBuf, recursive: bool) -> Result<()> {
    if let Err(e) = builder.append_dir(&target, &target) {
        return Err(ToteError::Archiver(e.to_string()))
    }
    for entry in target.read_dir().unwrap() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_dir() && recursive {
                process_dir(builder, e.path(), recursive)?
            } else if p.is_file() {
                process_file(builder, e.path())?
            }
        }
    }
    Ok(())
}

fn process_file<W: Write>(builder: &mut Builder<W>, target: PathBuf) -> Result<()> {
    if let Err(e) = builder.append_path(target) {
        Err(ToteError::Archiver(e.to_string()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
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
            let archiver = TarArchiver{};
            let inout = ArchiverOpts::create(PathBuf::from("results/test.tar"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(&inout);
            let path = PathBuf::from("results/test.tar");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::Tar);
            path
        });
    }

    
    #[test]
    fn test_targz() {
        run_test(|| {
            let archiver = TarGzArchiver{};
            let inout = ArchiverOpts::create(PathBuf::from("results/test.tar.gz"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(&inout);
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
            let archiver = TarBz2Archiver{};
            let inout = ArchiverOpts::create(PathBuf::from("results/test.tar.bz2"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(&inout);
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
            let archiver = TarXzArchiver{};
            let inout = ArchiverOpts::create(PathBuf::from("results/test.tar.xz"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(&inout);
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
            let archiver = TarZstdArchiver{};
            let inout = ArchiverOpts::create(PathBuf::from("results/test.tar.zst"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(&inout);
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
