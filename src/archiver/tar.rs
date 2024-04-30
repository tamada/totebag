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