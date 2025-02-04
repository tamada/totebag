//! This module provides an interface and struct for archiving the files.
//! The supported formats are: `cab`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
//! `lha` and `rar` formats are not supported for archiving.
//!
//! # Example: archiving the files
//!
//! ```
//! let archiver = Archiver::builder()
//!     .archive_file("destination/test.zip") // destination file.
//!     .targets(vec!["src", "Cargo.toml"])   // files to be archived.
//!     .rebase_dir(PathBuf::from("new"))     // rebased directory in the archive file.
//!     .overwrite(true)                      // set overwrite flag of the destination file.
//!     .build();
//! match archiver.perform() {
//!     Ok(_) => println!("archiving is done"),
//!     Err(e) => eprintln!("error: {:?}", e),
//! }
//! ```
use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

use ignore::{Walk, WalkBuilder};
use typed_builder::TypedBuilder;

use crate::format::{self, Format};
use crate::{IgnoreType, Result, ToteError};

mod cab;
mod lha;
mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// The trait for creating an archive file.
/// If you want to support archiving for a new format, you need to implement the `ToteArchiver` trait.
/// Then, the call [`perform_with`](Archiver::perform_with) method of [`Archiver`].
pub trait ToteArchiver {
    /// Perform the archiving operation.
    /// - `file` is the destination file for the archive.
    /// - `tps` is the list of files to be archived.
    fn perform(&self, file: File, tps: Vec<TargetPath>) -> Result<()>;
    /// Returns true if this archiver is enabled.
    fn enable(&self) -> bool;
}

/// Archiver is a struct to handle the archiving operation.
/// ```
/// let archiver = Archiver::builder()
///     .archive_file(PathBuf::from("results/test.zip"))
///     .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
///     .overwrite(true)      // default is false
/// //    .no_recursive(true)  // default is false
/// //    .ignore_types(vec![IgnoreType::Ignore])  // default is [IgnoreType::Default]
///    .build();
/// match archiver.perform() {
///    Ok(_)  => println!("archiving is done"),
///    Err(e) => eprintln!("error: {:?}", e),
/// }
/// ```
#[derive(Debug, TypedBuilder)]
pub struct Archiver {
    #[builder(default = format::Manager::default())]
    pub manager: format::Manager,
    /// The destination file for archiving.
    #[builder(setter(into))]
    pub archive_file: PathBuf,
    /// The list of files to be archived.
    #[builder(setter(into))]
    pub targets: Vec<PathBuf>,
    /// the prefix directory for the each file into the archive files when `Some`
    #[builder(default = None, setter(strip_option, into))]
    pub rebase_dir: Option<PathBuf>,
    /// Overwrite flag for archive file. Default is false.
    #[builder(default = false)]
    pub overwrite: bool,
    /// By default (`false`), read files by traversing the each `targets`.
    /// If `true`, it archives the specified files in `targets`.
    #[builder(default = false)]
    pub no_recursive: bool,
    /// specifies the ignore types for traversing.
    #[builder(default = vec![IgnoreType::Default], setter(into))]
    pub ignore_types: Vec<IgnoreType>,
}

/// TargetPath is a helper struct to handle the target path for the archiving operation.
pub struct TargetPath<'a> {
    base_path: &'a PathBuf,
    opts: &'a Archiver,
}

impl<'a> TargetPath<'a> {
    pub(crate) fn new(target: &'a PathBuf, archiver: &'a Archiver) -> Self {
        Self {
            base_path: target,
            opts: archiver,
        }
    }

    /// Returns the destination path for the target file.
    pub fn dest_path(&self, target: &PathBuf) -> PathBuf {
        let t = target.clone();
        let r = self.dest_path_impl(target);
        log::debug!("dest_path({:?}) -> {:?}", t, r);
        r
    }

    fn dest_path_impl(&self, target: &PathBuf) -> PathBuf {
        if let Some(rebase) = &self.opts.rebase_dir {
            rebase.join(target)
        } else {
            target.to_path_buf()
        }
    }

    /// Returns the directory traversing walker for the given path of this instance.
    pub fn walker(&self) -> Walk {
        let mut builder = WalkBuilder::new(self.base_path);
        build_walker_impl(self.opts, &mut builder);
        builder.build()
    }
}

impl Archiver {
    pub fn perform(&self) -> Result<()> {
        let archiver = match create_archiver(&self.manager, &self.archive_file) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };
        self.perform_with(archiver)
    }

    pub fn perform_with(&self, archiver: Box<dyn ToteArchiver>) -> Result<()> {
        if !archiver.enable() {
            return Err(ToteError::UnsupportedFormat(format!(
                "{}: not support archiving",
                self.format().unwrap()
            )));
        }
        let paths = self
            .targets
            .iter()
            .map(|item| TargetPath::new(item, self))
            .collect::<Vec<TargetPath>>();

        log::info!("{:?}: {}", self.archive_file, self.archive_file.exists());
        if self.archive_file.exists() {
            if self.archive_file.is_dir() {
                return Err(ToteError::DestIsDir(self.archive_file.clone()));
            } else if self.archive_file.is_file() && !self.overwrite {
                return Err(ToteError::FileExists(self.archive_file.clone()));
            }
        }
        if let Some(parent) = self.archive_file.parent() {
            if !parent.exists() {
                if let Err(e) = create_dir_all(parent) {
                    return Err(ToteError::IO(e));
                }
            }
        }
        match File::create(&self.archive_file) {
            Ok(f) => archiver.perform(f, paths),
            Err(e) => Err(ToteError::IO(e)),
        }
    }

    /// Returns the destination file for the archive with opening it and create the parent directories.
    /// If the path for destination is a directory or exists and overwrite is false,
    /// this function returns an error.
    pub fn destination(&self) -> Result<File> {
        let p = self.archive_file.as_path();
        log::info!("{:?}: {}", p, p.exists());
        if p.exists() {
            if p.is_dir() {
                return Err(ToteError::DestIsDir(p.to_path_buf()));
            } else if p.is_file() && !self.overwrite {
                return Err(ToteError::FileExists(p.to_path_buf()));
            }
        }
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                if let Err(e) = create_dir_all(parent) {
                    return Err(ToteError::IO(e));
                }
            }
        }
        match File::create(&self.archive_file) {
            Ok(f) => Ok(f),
            Err(e) => Err(ToteError::IO(e)),
        }
    }

    pub fn format(&self) -> Option<&Format> {
        self.manager.find(&self.archive_file)
    }

    pub fn info(&self) -> String {
        format!(
            "Format: {}\nArchive File: {}\nTargets: {}",
            self.format().unwrap().name,
            self.archive_file.to_str().unwrap(),
            self.targets
                .iter()
                .map(|item| item.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn ignore_types(&self) -> Vec<IgnoreType> {
        if self.ignore_types.is_empty() {
            vec![
                IgnoreType::Ignore,
                IgnoreType::GitIgnore,
                IgnoreType::GitGlobal,
                IgnoreType::GitExclude,
            ]
        } else {
            let mut r = HashSet::<IgnoreType>::new();
            for &it in &self.ignore_types {
                if it == IgnoreType::Default {
                    r.insert(IgnoreType::Ignore);
                    r.insert(IgnoreType::GitIgnore);
                    r.insert(IgnoreType::GitGlobal);
                    r.insert(IgnoreType::GitExclude);
                } else {
                    r.insert(it);
                }
            }
            r.into_iter().collect()
        }
    }
}

fn build_walker_impl(opts: &Archiver, w: &mut WalkBuilder) {
    for it in opts.ignore_types() {
        match it {
            IgnoreType::Default => w
                .ignore(true)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true),
            IgnoreType::GitIgnore => w.git_ignore(true),
            IgnoreType::GitGlobal => w.git_global(true),
            IgnoreType::GitExclude => w.git_exclude(true),
            IgnoreType::Hidden => w.hidden(true),
            IgnoreType::Ignore => w.ignore(true),
        };
    }
}

fn create_archiver<P: AsRef<Path>>(m: &format::Manager, dest: P) -> Result<Box<dyn ToteArchiver>> {
    use crate::archiver::cab::CabArchiver;
    use crate::archiver::lha::LhaArchiver;
    use crate::archiver::rar::RarArchiver;
    use crate::archiver::sevenz::SevenZArchiver;
    use crate::archiver::tar::{
        TarArchiver, TarBz2Archiver, TarGzArchiver, TarXzArchiver, TarZstdArchiver,
    };
    use crate::archiver::zip::ZipArchiver;

    let dest = dest.as_ref();
    let format = m.find(dest);
    match format {
        Some(format) => match format.name.as_str() {
            "Cab" => Ok(Box::new(CabArchiver {})),
            "Lha" => Ok(Box::new(LhaArchiver {})),
            "Rar" => Ok(Box::new(RarArchiver {})),
            "SevenZ" => Ok(Box::new(SevenZArchiver {})),
            "Tar" => Ok(Box::new(TarArchiver {})),
            "TarBz2" => Ok(Box::new(TarBz2Archiver {})),
            "TarGz" => Ok(Box::new(TarGzArchiver {})),
            "TarXz" => Ok(Box::new(TarXzArchiver {})),
            "TarZstd" => Ok(Box::new(TarZstdArchiver {})),
            "Zip" => Ok(Box::new(ZipArchiver::new())),
            _ => Err(ToteError::UnknownFormat(format.to_string())),
        },
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
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .rebase_dir("rebasedir")
            .overwrite(true)
            .build();
        assert_eq!(PathBuf::from("results/test.zip"), archiver.archive_file);
        assert_eq!(
            vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")],
            archiver.targets
        );
        assert_eq!(true, archiver.overwrite);
        assert_eq!(false, archiver.no_recursive);
        assert_eq!(1, archiver.ignore_types.len());
        assert_eq!(
            r#"Format: Zip
Archive File: results/test.zip
Targets: src, Cargo.toml"#,
            archiver.info()
        );
        assert!(archiver.destination().is_ok())
    }

    #[test]
    fn test_target_path() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            .rebase_dir(PathBuf::from("new"))
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("new/testdata/sample/src/archiver.rs").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/src/archiver.rs"))
        );
    }

    #[test]
    fn test_target_path2() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            //            .rebase_dir(None)
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("testdata/sample/Cargo.toml").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/Cargo.toml"))
        );
    }

    #[test]
    fn test_target_path3() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            .rebase_dir(PathBuf::from("new"))
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample/Cargo.toml");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("new/testdata/sample/Cargo.toml").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/Cargo.toml"))
        );
    }
}
