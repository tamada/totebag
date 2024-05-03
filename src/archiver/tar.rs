use std::io::Write;
use std::path::PathBuf;
use flate2::write::GzEncoder;
use bzip2::write::BzEncoder;
use tar::Builder;

use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::cli::{ToatError, Result};

pub(super) struct TarArchiver {
}
pub(super) struct TarGzArchiver {
}
pub(super) struct TarBz2Archiver {
}

impl Archiver for  TarArchiver {
    fn perform(&self, inout: ArchiverOpts) -> Result<()> {
        match inout.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                write_to_tar(file, inout.targets(), inout.recursive)
            }
        }
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}
impl Archiver for TarGzArchiver{
    fn perform(&self, inout: ArchiverOpts) -> Result<()> {
        match inout.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                let enc = GzEncoder::new(file, flate2::Compression::default());
                write_to_tar(enc, inout.targets(), inout.recursive)
            }
        }
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}
impl Archiver for  TarBz2Archiver {
    fn perform(&self, inout: ArchiverOpts) -> Result<()> {
        match inout.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                let enc = BzEncoder::new(file, bzip2::Compression::best());
                write_to_tar(enc, inout.targets(), inout.recursive)
            }
        }
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}

fn process_dir<W: Write>(builder: &mut Builder<W>, target: PathBuf, recursive: bool) -> Result<()> {
    if let Err(e) = builder.append_dir(&target, &target) {
        return Err(ToatError::ArchiverError(e.to_string()))
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
        Err(ToatError::ArchiverError(e.to_string()))
    } else {
        Ok(())
    }
}

fn write_to_tar<W: Write>(file: W, targets: Vec<PathBuf>, recursive: bool) -> Result<()> {
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
        return Err(ToatError::ArchiverError(e.to_string()))
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::archiver::Archiver;
    use crate::archiver::tar::{TarArchiver, TarGzArchiver, TarBz2Archiver};
    use crate::archiver::ArchiverOpts;
    use crate::format::Format;

    fn run_test<F>(f: F)
    where
        F: FnOnce() -> PathBuf,
    {
        // setup(); // 予めやりたい処理
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
            let result = archiver.perform(inout);
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
            let result = archiver.perform(inout);
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
            let result = archiver.perform(inout);
            let path = PathBuf::from("results/test.tar.bz2");
            assert!(result.is_ok());
            assert!(path.exists());
            assert_eq!(archiver.format(), Format::TarBz2);
            path
        });
    }

    fn teardown(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }
}
