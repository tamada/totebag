//! Archive format management module.
//! This module provides a way to manage archive formats.
//!
//! ## Examples
//!
//! As default, [Manager] has the following formats:
//! Cab, Lha, SevenZ, Rar, Tar, TarGz, TarBz2, TarXz, TarZstd, and Zip.
//!
//! ```
//! let manager = Manager::default();
//! let format = manager.find(PathBuf::from("test.zip"))
//!      .expect("Unexpected error: test.zip");
//! let format_name = format.name; // should be "Zip"
//! ```
//!
//! ## Use your own format
//!
//! ```
//! let mut manager = Manager::default();
//! let additional_format = Format::new("Compact Pro", vec![".sea", ".cpt"]);
//! manager.add(additional_format);
//! let format = manager.find("test.cpt")
//!     .expect("Unexpected error: test.cpt");
//! let format_name = format.name; // should be "Compact Pro"
//!
//! // remove the format
//! manager.remove(additional_format);
//! let _ = manager.find("test.cpt"); // should be None
//! ```
use std::fmt::Display;
use std::path::Path;

/// Archive format manager.
#[derive(Debug, Clone)]
pub struct Manager {
    formats: Vec<Format>,
}

impl Default for Manager {
    fn default() -> Self {
        Manager::new(vec![
            Format::new("Cab", vec![".cab"]),
            Format::new("Lha", vec![".lha", ".lzh"]),
            Format::new("SevenZ", vec![".7z"]),
            Format::new("Rar", vec![".rar"]),
            Format::new("Tar", vec![".tar"]),
            Format::new("TarGz", vec![".tar.gz", ".tgz"]),
            Format::new("TarBz2", vec![".tar.bz2", ".tbz2"]),
            Format::new("TarXz", vec![".tar.xz", ".txz"]),
            Format::new("TarZstd", vec![".tar.zst", ".tzst", ".tar.zstd", ".tzstd"]),
            Format::new("Zip", vec![".zip", ".jar", ".war", ".ear"]),
        ])
    }
}

impl Manager {
    pub fn new(formats: Vec<Format>) -> Self {
        Self { formats }
    }

    /// Returns `true` if all of the given file names are Some by [method.find] method.
    pub fn match_all<P: AsRef<Path>>(&self, args: &[P]) -> bool {
        args.iter().all(|p| self.find(p).is_some())
    }

    /// Find the format of the given file name.
    /// If the given file name has an unknown extension for totebag, it returns an `Err(ToteErro::Unknown)`.
    pub fn find<P: AsRef<Path>>(&self, path: P) -> Option<&Format> {
        let name = path
            .as_ref()
            .to_str()
            .expect("unexpected error: invalid path")
            .to_lowercase();
        self.formats.iter().find(|f| f.is_match(&name))
    }

    pub fn add(&mut self, format: Format) {
        self.formats.push(format);
    }

    pub fn remove(&mut self, format: Format) {
        self.formats.retain(|f| f != &format);
    }
}

/// Represents the archive format.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Format {
    pub name: String,
    exts: Vec<String>,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl AsRef<str> for Format {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl From<Format> for String {
    fn from(f: Format) -> Self {
        f.name
    }
}

impl Format {
    /// Create an instanceof Format with the name and its extensions.
    pub fn new<T: Into<String>>(name: T, exts: Vec<T>) -> Self {
        Self {
            name: name.into(),
            exts: exts.into_iter().map(|e| e.into().to_lowercase()).collect(),
        }
    }

    /// Returns `true` if the given file name has the extension of this format.
    pub fn is_match<P: AsRef<Path>>(&self, p: P) -> bool {
        let p = p.as_ref();
        let name = p.to_str().unwrap().to_lowercase();
        for ext in &self.exts {
            if name.ends_with(ext) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let manager = Manager::default();
        assert_eq!(manager.find("hoge.unknown"), None);
        assert_eq!(manager.find("test.cab"), Some(&manager.formats[0]));
        assert_eq!(manager.find("test.lha"), Some(&manager.formats[1]));
        assert_eq!(manager.find("test.lzh"), Some(&manager.formats[1]));
        assert_eq!(manager.find("test.7z"), Some(&manager.formats[2]));
        assert_eq!(manager.find("test.rar"), Some(&manager.formats[3]));
        assert_eq!(manager.find("test.tar"), Some(&manager.formats[4]));
        assert_eq!(manager.find("test.tar.gz"), Some(&manager.formats[5]));
        assert_eq!(manager.find("test.tgz"), Some(&manager.formats[5]));
        assert_eq!(manager.find("test.tar.bz2"), Some(&manager.formats[6]));
        assert_eq!(manager.find("test.tbz2"), Some(&manager.formats[6]));
        assert_eq!(manager.find("test.tar.xz"), Some(&manager.formats[7]));
        assert_eq!(manager.find("test.txz"), Some(&manager.formats[7]));
        assert_eq!(manager.find("test.tar.zst"), Some(&manager.formats[8]));
        assert_eq!(manager.find("test.tzst"), Some(&manager.formats[8]));
        assert_eq!(manager.find("test.tar.zstd"), Some(&manager.formats[8]));
        assert_eq!(manager.find("test.tzstd"), Some(&manager.formats[8]));
        assert_eq!(manager.find("test.zip"), Some(&manager.formats[9]));
        assert_eq!(manager.find("test.jar"), Some(&manager.formats[9]));
        assert_eq!(manager.find("test.ear"), Some(&manager.formats[9]));
        assert_eq!(manager.find("test.war"), Some(&manager.formats[9]));
    }

    #[test]
    fn test_is_all_args_archives() {
        let manager = Manager::default();
        assert!(manager.match_all(&[
            "test.zip",
            "test.tar",
            "test.tar.gz",
            "test.tgz",
            "test.tar.bz2",
            "test.tbz2",
            "test.rar",
        ]));
    }
}
