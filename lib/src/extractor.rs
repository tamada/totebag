//! This module provides the extractor for the archive file.
//! The supported formats are `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
//! 
//! # Example: listing the entries in the archive file
//! 
//! ```rust
//! use std::path::PathBuf;
//! 
//! let file = PathBuf::from("../testdata/test.zip");
//! let config = totebag::ListConfig::new(
//!     totebag::OutputFormat::Default,
//!     totebag::format::default_format_detector(),
//! );
//! match totebag::list(file, &config) {
//!     Ok(entries) => println!("{:?}", entries),
//!     Err(e) => println!("error: {:?}", e),
//! }
//! ```
//! 
//! # Example: extracting the archive file
//! 
//! The destination for extraction is the current directory in the following example.
//!
//! ```
//! use std::path::PathBuf;
//!
//! let config = totebag::ExtractConfig::builder()
//!     .dest("results")
//!     .build();
//! match totebag::extract("../testdata/test.zip", &config) {
//!     Ok(r) => println!("{:?}", r),
//!     Err(e) => println!("error: {:?}", e),
//! }
//! ```

use chrono::NaiveDateTime;
use serde::Serialize;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

use crate::format::Format;
use crate::{Result, ToteError};

mod cab;
mod cpio;
mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// This struct represents an entry in the archive file.
/// To build an instance of this struct, use [`Entry::new`] or [`Entry::builder`] methods in each [`ToteExtractor`].
///
/// # Example of builder
///
/// The required field is only [`name`](Entry::name), other fields are optional.
///
/// ```
/// use totebag::extractor::Entry;
///
/// let entry = Entry::builder()
///     .name("entry_name_extracted_from_archive_file")
///     .build();
/// ```
#[derive(Debug, TypedBuilder, Serialize)]
pub struct Entry {
    /// The path of the entry.
    #[builder(setter(into))]
    pub name: String,

    /// The compressed size of this entry.
    #[builder(setter(into, strip_option), default = None)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<u64>,

    /// The original size of this entry.
    #[builder(setter(into, strip_option), default = None)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_size: Option<u64>,

    /// The unix mode.
    #[builder(setter(into, strip_option), default = Some(0o644))]
    #[serde(
        serialize_with = "crate::outputs::serialize_option_u32_octal",
        skip_serializing_if = "Option::is_none"
    )]
    pub unix_mode: Option<u32>,

    /// The date of this entry.
    #[builder(setter(into), default = None)]
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Serialize)]
#[serde(rename = "archive-file")]
pub struct Entries {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
}

impl Entries {
    pub fn new(path: PathBuf, entries: Vec<Entry>) -> Self {
        Self { path, entries }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The trait for extracting the archive file.
/// If you want to support a new format for extraction, you need to implement the `ToteExtractor` trait.
/// Then, the call [`perform`](ToteExtractor::perform) and/or [`list`](ToteExtractor::list) method of [`ToteExtractor`].
pub trait ToteExtractor {
    /// returns the entry list of the given archive file.
    fn list(&self, archive_file: PathBuf) -> Result<Entries>;
    /// extract the given archive file into the specified directory with the given options.
    fn perform(&self, archive_file: PathBuf, opts: PathBuf) -> Result<()>;
}

/// Returns the extractor for the given archive file.
#[allow(dead_code)]
pub(super) fn create<P: AsRef<Path>>(file: P) -> Result<Box<dyn ToteExtractor>> {
    let file = file.as_ref();
    let binding = crate::format::default_format_detector();
    let format = binding.detect(file);
    create_with(file, format)
}

/// Returns the extractor for the given archive file.
/// The supported format is `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
pub(super) fn create_with<P: AsRef<Path>>(file: P, format: Option<&Format>) -> Result<Box<dyn ToteExtractor>> {
    let file = file.as_ref();
    match format {
        Some(format) => match format.name.as_str() {
            "Cab" => Ok(Box::new(cab::Extractor {})),
            "Cpio" => Ok(Box::new(cpio::Extractor {})),
            "Lha" => Ok(Box::new(lha::Extractor {})),
            "Rar" => Ok(Box::new(rar::Extractor {})),
            "SevenZ" => Ok(Box::new(sevenz::Extractor {})),
            "Tar" => Ok(Box::new(tar::Extractor {})),
            "TarBz2" => Ok(Box::new(tar::Bz2Extractor {})),
            "TarGz" => Ok(Box::new(tar::GzExtractor {})),
            "TarXz" => Ok(Box::new(tar::XzExtractor {})),
            "TarZstd" => Ok(Box::new(tar::ZstdExtractor {})),
            "Zip" => Ok(Box::new(zip::Extractor {})),
            s => Err(ToteError::UnknownFormat(format!("{s}: unknown format"))),
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
    fn test_destination1() {
        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts1 = crate::ExtractConfig::builder()
            .use_archive_name_dir(true)
            .build();
        let dest = opts1.dest(&archive_file).unwrap();
        assert_eq!(dest, PathBuf::from("./archive"));
    }

    #[test]
    fn test_destination2() {
        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts2 = crate::ExtractConfig::builder().build();
        let dest = opts2.dest(&archive_file).unwrap();
        assert_eq!(dest, PathBuf::from("."));
    }

    #[test]
    fn test_list_entries() {
        let archive_file = PathBuf::from("../testdata/test.zip");
        let extractor = create(&archive_file).unwrap();
        let entries = extractor.list(archive_file).unwrap();
        assert_eq!(entries.len(), 19);
    }

    #[test]
    fn test_list_entries_for_camouflaged_archive() {
        let archive_file = PathBuf::from("../testdata/camouflage_of_zip.rar");
        let format = crate::format::find_format_by_ext(".zip");
        let extractor = create_with(&archive_file, format).unwrap();
        let entries = extractor.list(archive_file).unwrap();
        assert_eq!(entries.len(), 19);
    }
}
