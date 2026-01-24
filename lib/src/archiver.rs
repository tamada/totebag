//! This module provides an interface and struct for archiving the files.
//! The supported formats are: `cab`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
//! `lha` and `rar` formats are not supported for archiving.
//!
//! # Example: archiving the files
//!
//! ```
//! use std::path::PathBuf;
//!
//! let config = totebag::ArchiveConfig::builder()
//!     .dest("results/test.zip")                  // destination file.
//!     .rebase_dir(PathBuf::from("new"))          // rebased directory in the archive file.
//!     .overwrite(true)                           // set overwrite flag of the destination file.
//!     .build();
//! let targets = vec!["src", "Cargo.toml"].iter() // files to be archived.
//!     .map(|s| PathBuf::from(s)).collect::<Vec<PathBuf>>();   
//! match totebag::archive(&targets, &config) {
//!     Ok(_) => println!("archiving is done"),
//!     Err(e) => eprintln!("error: {:?}", e),
//! }
//! ```
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::format::default_format_detector;
use crate::{Result, ToteError};

mod cab;
mod cpio;
mod lha;
mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// Represents a set of entries for archiving into the resultant file.
#[derive(Debug)]
pub struct ArchiveEntries {
    pub archive_file: PathBuf,
    pub entries: Vec<ArchiveEntry>,
    /// resultant file size (compressed size).
    pub compressed: u64,
}

/// Represents each entry in the archive file.
#[derive(Debug)]
pub struct ArchiveEntry {
    /// the source path of the entry.
    pub path: PathBuf,
    /// original file size.
    pub size: u64,
}

impl ArchiveEntries {
    pub fn new<P: AsRef<Path>>(path: P, entries: Vec<ArchiveEntry>, compressed: u64) -> Self {
        Self {
            archive_file: path.as_ref().to_path_buf(),
            entries,
            compressed,
        }
    }

    /// Returns the total file size of the entries.
    pub fn total(&self) -> u64 {
        self.entries.iter().map(|e| e.size).sum()
    }

    /// the length of the entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the entries is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ArchiveEntry {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size }
    }

    /// Create an instance of ArchiveEntry from the given path by obtaining the file size.
    pub fn from<P: AsRef<Path>>(p: P) -> Self {
        let path = p.as_ref().to_path_buf();
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        Self::new(path, size)
    }
}

/// The trait for creating an archive file.
/// If you want to support archiving for a new format, you need to implement the `ToteArchiver` trait.
/// Then, the call [`perform`](ToteArchiver::perform) method of [`ToteArchiver`].
pub trait ToteArchiver {
    /// Perform the archiving operation.
    /// - `file` is the destination file for the archive.
    /// - `tps` is the list of files to be archived.
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>>;

    /// Returns true if this archiver is enabled.
    fn enable(&self) -> bool;
}

pub(crate) fn collect_entries<P: AsRef<Path>>(targets: &[P], config: &crate::ArchiveConfig) -> Vec<PathBuf> {
    let mut r = vec![];
    for path in targets {
        for entry in config.iter(path) {
            let path = entry.into_path();
            if path.is_file() {
                r.push(path)
            }
        }
    }
    r
}

pub fn create<P: AsRef<Path>>(dest: P) -> Result<Box<dyn ToteArchiver>> {
    use crate::archiver::*;

    let dest = dest.as_ref();
    let fd = default_format_detector();
    let format = fd.detect(dest);
    match format {
        Some(format) => {
            let archiver: Box<dyn ToteArchiver> = match format.name.as_str() {
                "Cab" => Box::new(cab::Archiver {}),
                "Cpio" => Box::new(cpio::Archiver {}),
                "Lha" => Box::new(lha::Archiver {}),
                "Rar" => Box::new(rar::Archiver {}),
                "SevenZ" => Box::new(sevenz::Archiver {}),
                "Tar" => Box::new(tar::Archiver {}),
                "TarBz2" => Box::new(tar::Bz2Archiver {}),
                "TarGz" => Box::new(tar::GzArchiver {}),
                "TarXz" => Box::new(tar::XzArchiver {}),
                "TarZstd" => Box::new(tar::ZstdArchiver {}),
                "Zip" => Box::new(zip::Archiver::new()),
                _ => {
                    return Err(ToteError::UnknownFormat(format!(
                        "{}: unknown format",
                        format.name
                    )));
                }
            };
            if !archiver.enable() {
                Err(ToteError::UnsupportedFormat(format!(
                    "{}: unsupported format (archiving)",
                    format.name
                )))
            } else {
                Ok(archiver)
            }
        }
        None => Err(ToteError::Archiver(format!(
            "{:?}: no suitable archiver",
            dest.file_name().unwrap()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archiver() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("rebasedir")
            .overwrite(true)
            .build();
        if let Ok(p) = config.dest_file() {
            assert_eq!(PathBuf::from("results/test.zip"), p);
        }
        assert_eq!(true, config.overwrite);
        assert_eq!(false, config.no_recursive);
        assert_eq!(1, config.ignore.len());
        assert!(config.dest_file().is_ok())
    }

    #[test]
    fn test_target_path() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("new")
            .overwrite(true)
            .build();

        assert_eq!(
            PathBuf::from("new/testdata/sample/src/archiver.rs").as_path(),
            config.path_in_archive("testdata/sample/src/archiver.rs")
        );
    }

    #[test]
    fn test_target_path2() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            //            .rebase_dir(None)
            .overwrite(true)
            .build();

        assert_eq!(
            PathBuf::from("testdata/sample/Cargo.toml").as_path(),
            config.path_in_archive("testdata/sample/Cargo.toml")
        );
    }

    #[test]
    fn test_target_path3() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("new")
            .overwrite(true)
            .build();
        assert_eq!(
            PathBuf::from("new/testdata/sample/Cargo.toml").as_path(),
            config.path_in_archive("testdata/sample/Cargo.toml")
        );
    }
}
