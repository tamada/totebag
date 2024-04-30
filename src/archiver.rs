use std::fs::File;
use std::path::PathBuf;

use crate::cli::{ToatError, Result};
use crate::format::{find_format, Format};
use crate::archiver::zip::ZipArchiver;
use crate::archiver::rar::RarArchiver;
use crate::archiver::tar::{TarArchiver, TarGzArchiver, TarBz2Archiver};
use crate::verboser::{create_verboser, Verboser};
use crate::CliOpts;

mod zip;
mod rar;
mod tar;

pub trait Archiver {
    fn perform(&self, inout: ArchiverOpts) -> Result<()>;
    fn format(&self) -> Format;
}

pub fn create_archiver(dest: PathBuf) -> Result<Box<dyn Archiver>> {
    let format = find_format(dest.file_name());
    match format {
        Ok(format) => {
            return match format {
                Format::Zip => Ok(Box::new(ZipArchiver{})),
                Format::Tar => Ok(Box::new(TarArchiver{})),
                Format::TarGz => Ok(Box::new(TarGzArchiver{})),
                Format::TarBz2 => Ok(Box::new(TarBz2Archiver{})),
                Format::Rar => Ok(Box::new(RarArchiver{})),
                _ => Err(ToatError::UnsupportedFormat("unsupported format".to_string())),
            }
        }
        Err(msg) => Err(msg),
    }
}

pub fn archiver_info(archiver: &Box<dyn Archiver>, opts: &ArchiverOpts) -> String {
    format!(
        "Format: {:?}\nDestination: {:?}\nTargets: {:?}",
        archiver.format(),
        opts.destination(),
        opts.targets().iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>().join(", ")
    )
}

pub struct ArchiverOpts {
    pub dest: PathBuf,
    pub targets: Vec<PathBuf>,
    pub overwrite: bool,
    pub recursive: bool,
    pub v: Box<dyn Verboser>,
}

impl ArchiverOpts {
    pub fn new(opts: &CliOpts) -> Self {
        let args = opts.args.clone();
        let dest = opts.dest.clone().unwrap_or_else(|| {
            PathBuf::from(".")
        });
        ArchiverOpts {
            dest: dest,
            targets: args,
            overwrite: opts.overwrite,
            recursive: !opts.no_recursive,
            v: create_verboser(opts.verbose),
        }
    }

    #[cfg(test)]
    pub fn create(dest: PathBuf, targets: Vec<PathBuf>, overwrite: bool, recursive: bool, verbose: bool) -> Self {
        ArchiverOpts { dest, targets, overwrite, recursive, v: create_verboser(verbose) }
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        self.targets.clone()
    }
    pub fn destination(&self) -> Result<File> {
        let p = self.dest.as_path();
        if p.is_file() && p.exists() && !self.overwrite {
            return Err(ToatError::FileExists(self.dest.clone()))
        }
        match File::create(self.dest.as_path()) {
            Err(e) => Err(ToatError::IOError(e)),
            Ok(f) => Ok(f),
        }
    }
}
