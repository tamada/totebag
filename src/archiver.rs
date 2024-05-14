use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use crate::archiver::rar::RarArchiver;
use crate::archiver::sevenz::SevenZArchiver;
use crate::archiver::tar::{TarArchiver, TarBz2Archiver, TarGzArchiver, TarXzArchiver};
use crate::archiver::zip::ZipArchiver;
use crate::cli::{Result, ToteError};
use crate::format::{find_format, Format};
use crate::verboser::{create_verboser, Verboser};
use crate::CliOpts;

mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub trait Archiver {
    fn perform(&self, inout: &ArchiverOpts) -> Result<()>;
    fn format(&self) -> Format;
}

pub fn create_archiver(dest: &PathBuf) -> Result<Box<dyn Archiver>> {
    let format = find_format(dest.file_name());
    match format {
        Ok(format) => {
            return match format {
                Format::Zip => Ok(Box::new(ZipArchiver {})),
                Format::Tar => Ok(Box::new(TarArchiver {})),
                Format::TarGz => Ok(Box::new(TarGzArchiver {})),
                Format::TarBz2 => Ok(Box::new(TarBz2Archiver {})),
                Format::TarXz => Ok(Box::new(TarXzArchiver {})),
                Format::Rar => Ok(Box::new(RarArchiver {})),
                Format::SevenZ => Ok(Box::new(SevenZArchiver {})),
                _ => Err(ToteError::UnknownFormat(format.to_string())),
            }
        }
        Err(msg) => Err(msg),
    }
}

pub fn archiver_info(archiver: &Box<dyn Archiver>, opts: &ArchiverOpts) -> String {
    format!(
        "Format: {:?}\nDestination: {:?}\nTargets: {:?}",
        archiver.format(),
        opts.dest_path(),
        opts.targets()
            .iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>()
            .join(", ")
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
        let dest = opts.output.clone().unwrap_or_else(|| PathBuf::from("."));
        ArchiverOpts {
            dest: dest,
            targets: args,
            overwrite: opts.overwrite,
            recursive: !opts.no_recursive,
            v: create_verboser(opts.verbose),
        }
    }

    #[cfg(test)]
    pub fn create(
        dest: PathBuf,
        targets: Vec<PathBuf>,
        overwrite: bool,
        recursive: bool,
        verbose: bool,
    ) -> Self {
        ArchiverOpts {
            dest,
            targets,
            overwrite,
            recursive,
            v: create_verboser(verbose),
        }
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        self.targets.clone()
    }

    /// Simply return the path for destination.
    pub fn dest_path(&self) -> PathBuf {
        self.dest.clone()
    }

    /// Returns the destination file for the archive with opening it and create the parent directories.
    /// If the path for destination is a directory or exists and overwrite is false,
    /// this function returns an error.
    pub fn destination(&self) -> Result<File> {
        let p = self.dest.as_path();
        print!("{:?}: {}\n", p, p.exists());
        if p.is_file() && p.exists() && !self.overwrite {
            return Err(ToteError::FileExists(self.dest.clone()));
        }
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                if let Err(e) = create_dir_all(parent) {
                    return Err(ToteError::IOError(e));
                }
            }
        }
        match File::create(self.dest.as_path()) {
            Ok(f) => Ok(f),
            Err(e) => Err(ToteError::IOError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archiver() {
        let a1 = create_archiver(&PathBuf::from("results/test.tar"));
        assert!(a1.is_ok());
        assert_eq!(a1.unwrap().format(), Format::Tar);

        let a2 = create_archiver(&PathBuf::from("results/test.tar.gz"));
        assert!(a2.is_ok());
        assert_eq!(a2.unwrap().format(), Format::TarGz);

        let a3 = create_archiver(&PathBuf::from("results/test.tar.bz2"));
        assert!(a3.is_ok());
        assert_eq!(a3.unwrap().format(), Format::TarBz2);

        let a4 = create_archiver(&PathBuf::from("results/test.zip"));
        assert!(a4.is_ok());
        assert_eq!(a4.unwrap().format(), Format::Zip);

        let a5 = create_archiver(&PathBuf::from("results/test.rar"));
        assert!(a5.is_ok());
        assert_eq!(a5.unwrap().format(), Format::Rar);
    }
}
