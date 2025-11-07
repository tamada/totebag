/*!
 * This module provides the extractor for the archive file.
 * The supported formats are `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
 *
 * # Example: listing the entries in the archive file
 *
 * ```
 * let file = PathBuf::from("testdata/test.zip");
 * let extractor = Extractor::builder()
 *     .archive_file(file)
 *     .build();
 * match extractor.list() {
 *     Ok(entries) => println!("{:?}", entries),
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 *
 * # Example: extracting the archive file
 *
 * The destination for extraction is the current directory in the following example.
 *
 * ```
 * let extractor = Extractor::builder()
 *     .archive_file(PathBuf::From("testdata/test.zip"))
 *     .build();
 * match extractor.perform(&opts) {
 *     Ok(r) => println!("{:?}", r),
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 */
use chrono::NaiveDateTime;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

use crate::format::Format;
use crate::{Result, ToteError};

mod cab;
mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// This struct represents an entry in the archive file.
/// To build an instance of this struct, use [`Entry::new`] or [`Entry::builder`] methods.
///
/// # Example of builder
///
/// The required field is only [`name`](Entry::name), other fields are optional.
///
/// ```
/// let entry = Entry::builder()
///     .name("entry_name_extracted_from_archive_file")
///     .build();
/// ```
#[derive(Debug, TypedBuilder)]
pub struct Entry {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option))]
    pub compressed_size: Option<u64>,
    #[builder(setter(into, strip_option))]
    pub original_size: Option<u64>,
    #[builder(setter(into, strip_option))]
    pub unix_mode: Option<u32>,
    #[builder(setter(into, strip_option))]
    pub date: Option<NaiveDateTime>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Entry {
    pub fn new(
        name: String,
        compressed_size: Option<u64>,
        original_size: Option<u64>,
        unix_mode: Option<u32>,
        date: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            name,
            compressed_size,
            original_size,
            unix_mode,
            date,
        }
    }
}

/// This struct provides the utility functions for the extraction paths of files in the archive file.
pub struct PathUtils<'a> {
    e: &'a Extractor,
}

impl PathUtils<'_> {
    /// Returns the base directory for the archive file extract to.
    pub fn base_dir(&self) -> PathBuf {
        self.e.base_dir()
    }

    /// Returns the path of the `target` file in the archive file for output.
    pub fn destination<P: AsRef<Path>>(&self, target: P) -> Result<PathBuf> {
        self.e.destination(target)
    }
}

/// This struct represents the extractor for the archive file.
#[derive(Debug, TypedBuilder)]
pub struct Extractor {
    #[builder(default = crate::format::Manager::default())]
    pub manager: crate::format::Manager,

    #[builder(setter(into))]
    /// The archive file for extraction.
    pub archive_file: PathBuf,

    /// The destination directory for the extracted files.
    /// The default is the current directory.
    #[builder(default = PathBuf::from("."), setter(into))]
    pub destination: PathBuf,

    /// If true, the destination directory is the result of `destination` joined the stem of `archive_file`.
    /// For example, if `archive_file` is `/tmp/test.zip`, the `destination` is `/tmp/archive`,
    /// the resultant destination directory is `/tmp/archive/test`.
    #[builder(default = false)]
    pub use_archive_name_dir: bool,

    /// If true, it overwrite the existing file in the destination directory.
    #[builder(default = false)]
    pub overwrite: bool,
}

impl Extractor {
    /// Returns the entries in the archive file.
    pub fn list(&self) -> Result<Vec<Entry>> {
        let extractor = create(&self.manager, &self.archive_file)?;
        self.list_with(extractor)
    }

    /// Returns the entries in the archive file with the given extractor.
    pub fn list_with(&self, extractor: Box<dyn ToteExtractor>) -> Result<Vec<Entry>> {
        extractor.list(self.archive_file.clone())
    }

    /// Execute extraction of the archive file.
    pub fn perform(&self) -> Result<()> {
        let extractor = create(&self.manager, &self.archive_file)?;
        self.perform_with(extractor)
    }

    /// Execute extraction of the archive file with the given extractor.
    pub fn perform_with(&self, extractor: Box<dyn ToteExtractor>) -> Result<()> {
        match self.can_extract() {
            Ok(_) => extractor.perform(self.archive_file.clone(), PathUtils { e: self }),
            Err(e) => Err(e),
        }
    }

    /// Returns the information of the extractor.
    pub fn info(&self) -> String {
        format!(
            "Format: {}\nFile: {:?}\nDestination: {:?}",
            self.manager.find(&self.archive_file).unwrap(),
            self.archive_file,
            self.destination,
        )
    }

    pub fn format(&self) -> Option<&Format> {
        self.manager.find(&self.archive_file)
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    fn base_dir(&self) -> PathBuf {
        if self.use_archive_name_dir {
            if let Some(stem) = self.archive_file.file_stem() {
                self.destination.join(stem)
            } else {
                self.destination.clone()
            }
        } else {
            self.destination.clone()
        }
    }

    /// Return the path of the `target` file for output.
    fn destination<P: AsRef<Path>>(&self, target: P) -> Result<PathBuf> {
        let base = self.base_dir();
        let dest = base.join(target);
        if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest.clone()))
        } else {
            Ok(dest)
        }
    }

    pub fn can_extract(&self) -> Result<()> {
        let dest = self.base_dir();
        if dest == PathBuf::from(".") {
            Ok(())
        } else if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest))
        } else {
            Ok(())
        }
    }
}

/// The trait for extracting the archive file.
/// If you want to support a new format for extraction, you need to implement the `ToteExtractor` trait.
/// Then, the call [`perform_with`](Extractor::perform_with) and/or [`list_with`](Extractor::list_with) method of [`Extractor`].
pub trait ToteExtractor {
    /// returns the entry list of the given archive file.
    fn list(&self, archive_file: PathBuf) -> Result<Vec<Entry>>;
    /// extract the given archive file into the specified directory with the given options.
    fn perform(&self, archive_file: PathBuf, opts: PathUtils) -> Result<()>;
}

/// Returns the extractor for the given archive file.
/// The supported format is `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
fn create<P: AsRef<Path>>(m: &crate::format::Manager, file: P) -> Result<Box<dyn ToteExtractor>> {
    let file = file.as_ref();
    let format = m.find(file);
    match format {
        Some(format) => match format.name.as_str() {
            "Cab" => Ok(Box::new(cab::CabExtractor {})),
            "Lha" => Ok(Box::new(lha::LhaExtractor {})),
            "Rar" => Ok(Box::new(rar::RarExtractor {})),
            "SevenZ" => Ok(Box::new(sevenz::SevenZExtractor {})),
            "Tar" => Ok(Box::new(tar::TarExtractor {})),
            "TarBz2" => Ok(Box::new(tar::TarBz2Extractor {})),
            "TarGz" => Ok(Box::new(tar::TarGzExtractor {})),
            "TarXz" => Ok(Box::new(tar::TarXzExtractor {})),
            "TarZstd" => Ok(Box::new(tar::TarZstdExtractor {})),
            "Zip" => Ok(Box::new(zip::ZipExtractor {})),
            s => Err(ToteError::UnknownFormat(format!(
                "{s}: unsupported format",
            ))),
        },
        None => Err(ToteError::Extractor(format!(
            "{file:?} no suitable extractor"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination() {
        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts1 = Extractor::builder()
            .archive_file(archive_file)
            .use_archive_name_dir(true)
            .build();
        assert_eq!(opts1.base_dir(), PathBuf::from("./archive"));
        if let Ok(t) = opts1.destination("text1.txt") {
            assert_eq!(t, PathBuf::from("./archive/text1.txt"));
        }
        if let Ok(t) = opts1.destination("text2.txt") {
            assert_eq!(t, PathBuf::from("./archive/text2.txt"));
        }

        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts2 = Extractor::builder().archive_file(archive_file).build();
        assert_eq!(opts2.base_dir(), PathBuf::from("."));
        if let Ok(t) = opts2.destination("./text1.txt") {
            assert_eq!(t, PathBuf::from("./text1.txt"));
        }
    }
}
