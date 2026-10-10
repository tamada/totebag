//! This module provides the extractor for the archive file.
//! The supported formats are `ar`, `cab`, `cpio`, `lha`, `7z`, `tar`, `tar.gz`,
//! `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`, plus `rar` when the `rar` feature
//! is enabled.
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
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

use crate::format::Format;
use crate::{Error, Result};

mod ar;
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
/// See the module documentation for the list of supported formats.
pub(super) fn create_with<P: AsRef<Path>>(
    file: P,
    format: Option<&Format>,
) -> Result<Box<dyn ToteExtractor>> {
    let file = file.as_ref();
    match format {
        Some(format) => match format.name.as_str() {
            "Ar" => Ok(Box::new(ar::Extractor {})),
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
            s => Err(Error::UnknownFormat(format!("{s}: unknown format"))),
        },
        None => Err(Error::Extractor(format!("{file:?} no suitable extractor"))),
    }
}

/// Opens `archive_file` for reading through a `BufReader`, so that formats
/// that read small headers do not issue a system call for each one.
pub(crate) fn buf_open(archive_file: &Path) -> Result<BufReader<File>> {
    let r = File::open(archive_file).map_err(Error::IO)?;
    Ok(BufReader::new(r))
}

/// The destination of the entry `name` under `base`, or an error when the name
/// would leave `base`: a `..` component, an absolute path, or a drive prefix.
/// `.` components are dropped.
pub(crate) fn safe_join(base: &Path, name: &Path) -> Result<PathBuf> {
    use std::path::Component;
    let mut rel = PathBuf::new();
    for component in name.components() {
        match component {
            Component::Normal(part) => rel.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::UnsafePath(name.to_path_buf()));
            }
        }
    }
    Ok(base.join(rel))
}

/// [`safe_join`] for formats whose names use DOS `\` separators (cab), so that
/// `dir\file` becomes `dir/file` rather than one file named `dir\file`.
pub(crate) fn safe_join_dos(base: &Path, name: &str) -> Result<PathBuf> {
    safe_join(base, Path::new(&name.replace('\\', "/")))
}

pub(crate) fn create_parent_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(Error::IO)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_with() {
        let r = create_with(PathBuf::from("../testdata/test.zip"), None);
        assert!(r.is_err());
    }

    #[test]
    fn test_create_with_unknown_format() {
        let format = Format::new("Hoge", vec![".hoge"]);
        let r = create_with(PathBuf::from("../testdata/test.zip"), Some(&format));
        assert!(r.is_err());
    }

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

    #[test]
    fn test_safe_join() {
        let base = PathBuf::from("base");
        assert_eq!(
            safe_join(&base, Path::new("a")).unwrap(),
            PathBuf::from("base/a")
        );
        assert_eq!(
            safe_join(&base, Path::new("./a/./b.txt")).unwrap(),
            PathBuf::from("base/a/b.txt")
        );
        for name in [
            "../evil.txt",
            "a/../../evil.txt",
            "a/../b.txt",
            "/etc/passwd",
        ] {
            assert!(
                matches!(
                    safe_join(&base, Path::new(name)),
                    Err(Error::UnsafePath(..))
                ),
                "{name} must be rejected"
            );
        }
    }

    #[test]
    fn test_safe_join_dos() {
        let base = Path::new("out");
        assert_eq!(
            safe_join_dos(base, "dir\\file.txt").unwrap(),
            PathBuf::from("out/dir/file.txt")
        );
        assert!(matches!(
            safe_join_dos(base, "..\\evil.txt"),
            Err(Error::UnsafePath(..))
        ));
    }
}
