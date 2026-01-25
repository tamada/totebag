//! Archive format management module.
//! This module detects archive formats from file.
//! 
//! ## Format Detection Strategies
//! 
//! `totebag` provides three strategies to detect the archive format of a file:
//! 
//! 1. By file extension (default)
//! 2. By magic number (file signature)
//! 3. Fixed format (forcing a specific format)
//! 
//! ### By File Extension
//! 
//! This is the default strategy used by `totebag`.
//! It detects the archive format based on the file extension.
//! For example, a file with the extension `.zip` is recognized as a Zip archive.
//! 
//! ```rust
//! use std::path::PathBuf;
//! let fd = totebag::format::default_format_detector();
//! let format = fd.detect(&PathBuf::from("../testdata/test.zip"))
//!     .expect("zip format should be recognized by its file extension");
//! ```
//! 
//! ### By Magic Number
//! 
//! This strategy detects the archive format by reading the file's magic number (file signature). 
//! This method is more reliable than using file extensions, as it examines the actual content of the file.
//! However, it may be slightly slower due to the need to read the file.
//! Additionally, this method cannot distinguish just `.gz` file and `tar.gz` file.
//! See [infer](https://docs.rs/infer/latest/infer/) crate's documentation for more details about supported formats by magic number.
//! 
//! ```rust
//! use std::path::PathBuf;
//! let fd = totebag::format::magic_number_format_detector();
//! let format = fd.detect(&PathBuf::from("../testdata/test.rar"))
//!     .expect("rar format should be recognized by its magic number");
//! ```
//! 
//! ### Fixed format
//! 
//! This strategy forces `totebag` to treat the file as a specific archive format, regardless of its extension or content.
//! This can be useful when dealing with files that have incorrect extensions or when you want to override the default format detection behavior.
//! 
//! ```rust
//! use std::path::PathBuf;
//! let fd = totebag::format::fixed_format_detector(
//!     totebag::format::find_format_by_name("Rar").unwrap()
//! );
//! let format = fd.detect(&PathBuf::from("../testdata/test.zip"))
//!     .expect("this method always returns the fixed format (this example returns always rar format)");
//! ```
use std::fmt::Display;
use std::path::Path;
use std::sync::LazyLock;

static MANAGER: LazyLock<Manager> = LazyLock::new(Manager::default);

/// Archive format manager.
#[derive(Debug, Clone)]
struct Manager {
    formats: Vec<Format>,
}

/// Returns an instance of the format detector by file extension.
pub fn default_format_detector() -> Box<dyn FormatDetector> {
    Box::new(ExtensionFormatDetector {})
}

/// Returns an instance of the format detector by the magic number of file header.
pub fn magic_number_format_detector() -> Box<dyn FormatDetector> {
    Box::new(MagicNumberFormatDetector {})
}

/// Returns an instance of the format detector for the given format.
pub fn fixed_format_detector(format: &'static Format) -> Box<dyn FormatDetector> {
    Box::new(FixedFormatDetector::new(format))
}

/// The trait for detecting the archive format of a file.
pub trait FormatDetector {
    /// Detects the archive format of the given file path.
    /// Returns `Some(&`[`Format`]`)` if the format is recognized, otherwise returns `None`.
    fn detect(&self, path: &Path) -> Option<&Format>;
}

struct ExtensionFormatDetector;
struct MagicNumberFormatDetector;
struct FixedFormatDetector {
    format: &'static Format,
}

impl FixedFormatDetector {
    pub(crate) fn new(format: &'static Format) -> Self {
        Self { format }
    }
}

impl FormatDetector for MagicNumberFormatDetector {
    fn detect(&self, filename: &Path) -> Option<&Format> {
        match infer::get_from_path(filename) {
            Err(e) => {
                log::error!("Failed to read file for format detection: {e:?}");
                None
            },
            Ok(Some(info)) => {
                match info.mime_type() {
                    "application/x-archive" => find_format_by_name("Ar"),
                    "application/x-cab" => find_format_by_name("Cab"),
                    "application/x-cpio" => find_format_by_name("Cpio"),
                    "application/x-lzh" | "application/x-lha" => find_format_by_name("Lha"),
                    "application/x-7z-compressed" => find_format_by_name("SevenZ"),
                    "application/vnd.rar" => find_format_by_name("Rar"),
                    "application/x-tar" => find_format_by_name("Tar"),
                    "application/gzip" => find_format_by_name("TarGz"),
                    "application/x-bzip2" => find_format_by_name("TarBz2"),
                    "application/x-xz" => find_format_by_name("TarXz"),
                    "application/zstd" => find_format_by_name("TarZstd"),
                    "application/zip" | "application/java-archive" => find_format_by_name("Zip"),
                    other => {
                        log::error!("Unknown file format detected by magic number: {filename:?} (mime-type: {other})");
                        None
                    }
                }
            },
            Ok(None) => {
                log::error!("Could not detect file format from magic number: {filename:?}");
                None
            }
        }
    }
}

impl FormatDetector for ExtensionFormatDetector {
    fn detect(&self, path: &Path) -> Option<&Format> {
        MANAGER.formats.iter().find(|f| f.match_exts(path))
    }
}

impl FormatDetector for FixedFormatDetector {
    fn detect(&self, _path: &Path) -> Option<&Format> {
        Some(self.format)
    }
}

impl Default for Manager {
    fn default() -> Self {
        Manager::new(vec![
            Format::new("Ar", vec![".ar", ".a", ".lib"]),
            Format::new("Cab", vec![".cab"]),
            Format::new("Cpio", vec![".cpio"]),
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

/// Returns `true` if all of the given file names are Some by the given [`FormatDetector::detect`] method.
pub fn is_all_archive_file<P: AsRef<Path>>(args: &[P], fd: &dyn FormatDetector) -> bool {
    args.iter().all(|p| fd.detect(p.as_ref()).is_some())
}

/// Find the format by its name.
/// If the given name is unknown format for totebag, it returns `None`.
pub fn find_format_by_name<S: AsRef<str>>(name: S) -> Option<&'static Format> {
    let name = name.as_ref().to_lowercase();
    log::debug!("find format by name: {name}");
    MANAGER.formats.iter().find(|f| f.name.to_lowercase() == name)
}

/// Find the instance of [`Format`] from the given file extension.
pub fn find_format_by_ext<S: AsRef<str>>(ext: S) -> Option<&'static Format> {
    let ext = ext.as_ref();
    let ext = if !ext.starts_with('.') {
        format!(".{ext}")
    } else {
        ext.to_string()
    }.to_lowercase();
    MANAGER.formats.iter().find(|f| f.exts.contains(&ext))
}

impl Manager {
    pub(crate) fn new(formats: Vec<Format>) -> Self {
        Self { formats }
    }
}

/// Represents the archive format.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Format {
    /// The general format name.
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
    pub fn match_exts<P: AsRef<Path>>(&self, p: P) -> bool {
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
        use std::path::PathBuf;
        let fd = default_format_detector();
        assert_eq!(fd.detect(&PathBuf::from("hoge.unknown")), None);
        assert_eq!(fd.detect(&PathBuf::from("test.a")), Some(&MANAGER.formats[0]));
        assert_eq!(fd.detect(&PathBuf::from("test.ar")), Some(&MANAGER.formats[0]));
        assert_eq!(fd.detect(&PathBuf::from("test.lib")), Some(&MANAGER.formats[0]));
        assert_eq!(fd.detect(&PathBuf::from("test.cab")), Some(&MANAGER.formats[1]));
        assert_eq!(fd.detect(&PathBuf::from("test.cpio")), Some(&MANAGER.formats[2]));
        assert_eq!(fd.detect(&PathBuf::from("test.lha")), Some(&MANAGER.formats[3]));
        assert_eq!(fd.detect(&PathBuf::from("test.lzh")), Some(&MANAGER.formats[3]));
        assert_eq!(fd.detect(&PathBuf::from("test.7z")), Some(&MANAGER.formats[4]));
        assert_eq!(fd.detect(&PathBuf::from("test.rar")), Some(&MANAGER.formats[5]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar")), Some(&MANAGER.formats[6]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar.gz")), Some(&MANAGER.formats[7]));
        assert_eq!(fd.detect(&PathBuf::from("test.tgz")), Some(&MANAGER.formats[7]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar.bz2")), Some(&MANAGER.formats[8]));
        assert_eq!(fd.detect(&PathBuf::from("test.tbz2")), Some(&MANAGER.formats[8]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar.xz")), Some(&MANAGER.formats[9]));
        assert_eq!(fd.detect(&PathBuf::from("test.txz")), Some(&MANAGER.formats[9]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar.zst")), Some(&MANAGER.formats[10]));
        assert_eq!(fd.detect(&PathBuf::from("test.tzst")), Some(&MANAGER.formats[10]));
        assert_eq!(fd.detect(&PathBuf::from("test.tar.zstd")), Some(&MANAGER.formats[10]));
        assert_eq!(fd.detect(&PathBuf::from("test.tzstd")), Some(&MANAGER.formats[10]));
        assert_eq!(fd.detect(&PathBuf::from("test.zip")), Some(&MANAGER.formats[11]));
        assert_eq!(fd.detect(&PathBuf::from("test.jar")), Some(&MANAGER.formats[11]));
        assert_eq!(fd.detect(&PathBuf::from("test.ear")), Some(&MANAGER.formats[11]));
        assert_eq!(fd.detect(&PathBuf::from("test.war")), Some(&MANAGER.formats[11]));
    }

    #[test]
    fn test_is_all_args_archives() {
        let fd = default_format_detector();
        assert!(is_all_archive_file(&[
            "test.zip",
            "test.tar",
            "test.tar.gz",
            "test.tgz",
            "test.tar.bz2",
            "test.tbz2",
            "test.rar",
        ], fd.as_ref()));
    }

    #[test]
    fn test_find_by_name() {
        let format = find_format_by_name("zip").unwrap();
        assert_eq!(format.name, "Zip");
        let format = find_format_by_name("TaRZsTd").unwrap();
        assert_eq!(format.name, "TarZstd");
        let format = find_format_by_name("unknown");
        assert!(format.is_none());
    }

    #[test]
    fn test_find_by_ext() {
        let format = find_format_by_ext(".ZIP").unwrap();
        assert_eq!(format.name, "Zip");
        let format = find_format_by_ext("tAr.Gz").unwrap();
        assert_eq!(format.name, "TarGz");
        let format = find_format_by_ext(".unknown");
        assert!(format.is_none());
    }

    #[test]
    fn test_extension_format_detector() {
        let detector = ExtensionFormatDetector {};
        let format = detector.detect(Path::new("test.zip")).unwrap();
        assert_eq!(format.name, "Zip");
        let format = detector.detect(Path::new("test.tar.gz")).unwrap();
        assert_eq!(format.name, "TarGz");
        let format = detector.detect(Path::new("test.rar")).unwrap();
        assert_eq!(format.name, "Rar");
        let format = detector.detect(Path::new("test.unknown"));
        assert!(format.is_none());
    }

    #[test]
    fn test_magic_number_format_detector() {
        let detector = MagicNumberFormatDetector {};
        let format = detector.detect(Path::new("../testdata/test.zip")).unwrap();
        assert_eq!(format.name, "Zip");
        let format = detector.detect(Path::new("../testdata/test.tar.gz")).unwrap();
        assert_eq!(format.name, "TarGz");
        let format = detector.detect(Path::new("../testdata/test.rar")).unwrap();
        assert_eq!(format.name, "Rar");
        let format = detector.detect(Path::new("../testdata/camouflage_of_zip.rar")).unwrap();
        assert_eq!(format.name, "Zip");
        let format = detector.detect(Path::new("../testdata/not_exist_file.rar"));
        assert!(format.is_none());
    }

    #[test]
    fn test_fixed_format_detector() {
        let format = find_format_by_name("Zip").unwrap();
        let detector = FixedFormatDetector::new(format);
        let detected_format = detector.detect(Path::new("anyfile.anyext")).unwrap();
        assert_eq!(detected_format.name, "Zip");
    }
}
