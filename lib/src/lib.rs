//! # Totebag
//!
//! `totebag` is an archiving utilities that can archive and extract files supported several formats.
//!
pub mod archiver;
pub mod extractor;
pub mod format;
pub(crate) mod outputs;

/// Compiles the Rust code blocks in `README.md` as doctests, so the examples
/// there cannot drift away from the API (issue #81). Not part of the rendered
/// documentation.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use typed_builder::TypedBuilder;

use crate::archiver::ArchiveEntries;
use crate::extractor::Entries;
use crate::format::{FormatDetector, default_format_detector};

/// Define the result type for this library.
pub type Result<T> = std::result::Result<T, Error>;

/// Define the ignore types for directory traversing.
#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum IgnoreType {
    /// use `git-ignore`, `.gitglobal`, `.gitexclude`, and `.ignore`.
    Default,
    /// ignore hidden files and directories.
    Hidden,
    /// ignore files and directories that are listed in `.gitignore`.
    GitIgnore,
    /// ignore files and directories that are listed in `.gitglobal`.
    GitGlobal,
    /// ignore files and directories that are listed in `.gitexclude`.
    GitExclude,
    /// ignore files and directories that are listed in `.ignore`.
    Ignore,
}

/// Errors that can occur when using this library.
///
/// This enum represents all possible errors that can be returned
/// from archive and extraction operations.
#[derive(Debug)]
pub enum Error {
    /// Error from archiving operation with a descriptive message
    Archiver(String),
    /// Multiple errors occurred during an operation
    Array(Vec<Error>),
    /// The destination path is a directory when a file was expected
    DestIsDir(PathBuf),
    /// The target directory already exists
    DirExists(PathBuf),
    /// Error from extraction operation with a descriptive message
    Extractor(String),
    /// A fatal error from an underlying library
    Fatal(Box<dyn std::error::Error + Send + Sync>),
    /// The format is known but its support was not compiled in.
    ///
    /// `feature` names the Cargo feature that enables it.
    FeatureDisabled {
        format: String,
        feature: &'static str,
    },
    /// The specified file was not found
    FileNotFound(PathBuf),
    /// The file already exists when it shouldn't be overwritten
    FileExists(PathBuf),
    /// Standard I/O error
    IO(std::io::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// No arguments were provided when some were required
    NoArgumentsGiven,
    /// A warning message that doesn't halt execution
    Warn(String),
    /// The archive format could not be determined
    UnknownFormat(String),
    /// The format is recognized but not supported for the operation
    UnsupportedFormat(String),
    /// XML serialization/deserialization error
    Xml(serde_xml_rs::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Archiver(s) => write!(f, "Archiver error: {s}"),
            Error::Array(errs) => errs
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                .fmt(f),
            Error::DestIsDir(p) => write!(f, "{}: Destination is a directory", p.to_str().unwrap()),
            Error::DirExists(p) => write!(f, "{}: Directory already exists", p.to_str().unwrap()),
            Error::Extractor(s) => write!(f, "Extractor error: {s}"),
            Error::Fatal(e) => write!(f, "Error: {e}"),
            Error::FeatureDisabled { format, feature } => write!(
                f,
                "{format}: support is not compiled in (rebuild with --features {feature})"
            ),
            Error::FileNotFound(p) => write!(f, "{}: File not found", p.display()),
            Error::FileExists(p) => write!(f, "{}: File already exists", p.display()),
            Error::IO(e) => write!(f, "IO error: {e}"),
            Error::Json(e) => write!(f, "Json error: {e}"),
            Error::NoArgumentsGiven => write!(f, "No arguments given. Use --help for usage."),
            Error::Warn(s) => write!(f, "Unknown error: {s}"),
            Error::UnknownFormat(s) => write!(f, "{s}: Unknown format"),
            Error::UnsupportedFormat(s) => write!(f, "{s}: Unsupported format"),
            Error::Xml(e) => write!(f, "Xml error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fatal(e) => Some(e.as_ref()),
            Error::IO(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Xml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<serde_xml_rs::Error> for Error {
    fn from(e: serde_xml_rs::Error) -> Self {
        Error::Xml(e)
    }
}

impl Error {
    /// Builds the error reported when a format can be extracted but not created.
    ///
    /// Kept in one place so that [`archiver::create`] and the extract-only
    /// archiver stubs cannot drift apart.
    pub(crate) fn unsupported_for_archiving(format: &str) -> Self {
        Error::UnsupportedFormat(format!("{format} (archiving)"))
    }

    /// Returns `Ok(ok)` if there are no errors, otherwise returns an appropriate error.
    ///
    /// This is a helper method to consolidate multiple errors into a single result.
    ///
    /// # Arguments
    ///
    /// * `ok` - The success value to return if no errors occurred
    /// * `errs` - A vector of errors that may have occurred
    ///
    /// # Returns
    ///
    /// * `Ok(ok)` if `errs` is empty
    /// * `Err(error)` if `errs` contains a single error
    /// * `Err(Error::Array(errs))` if `errs` contains multiple errors
    pub fn error_or<T>(ok: T, errs: Vec<Self>) -> Result<T> {
        if errs.is_empty() {
            Ok(ok)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Error::Array(errs))
        }
    }

    /// Returns `Ok(ok())` if there are no errors, otherwise returns an appropriate error.
    /// see [`Error::error_or`] for details of error handling.
    pub fn error_or_else<F, O>(ok: F, errs: Vec<Self>) -> Result<O>
    where
        F: FnOnce() -> O,
    {
        if errs.is_empty() {
            Ok(ok())
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Error::Array(errs))
        }
    }
}

/// Extract an archive file to the specified destination directory.
///
/// # Arguments
///
/// * `archive_file` - The path to the archive file to extract
/// * `config` - The extraction configuration
///
/// # Examples
///
/// ```
/// use totebag::{extract, ExtractConfig};
///
/// let config = ExtractConfig::builder()
///     .dest("results/doc_extract")
///     .overwrite(true)
///     .build();
/// match extract("../testdata/test.zip", &config) {
///     Ok(_) => println!("Extraction successful"),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn extract<P: AsRef<Path>>(archive_file: P, config: &ExtractConfig) -> Result<()> {
    let archive_file = archive_file.as_ref();
    let base_dir = config.dest(archive_file)?;
    let extractor = config.extractor(archive_file)?;
    extractor.perform(archive_file.to_path_buf(), base_dir)
}

/// Configuration for extracting archive files.
///
/// This struct holds all the options needed to extract an archive file.
/// Use the builder pattern to create an instance.
///
/// # Examples
///
/// ```
/// use totebag::ExtractConfig;
/// use std::path::PathBuf;
///
/// let config = ExtractConfig::builder()
///     .dest(PathBuf::from("results/doc_extract"))
///     .overwrite(true)
///     .use_archive_name_dir(false)
///     .build();
/// ```
#[derive(TypedBuilder)]
pub struct ExtractConfig {
    /// The destination directory for extraction.
    #[builder(setter(into), default = PathBuf::from("."))]
    pub dest: PathBuf,
    /// Overwrite flag, if `true`, overwrite the files.
    #[builder(default = false)]
    pub overwrite: bool,
    /// If `true`, the destination path becomes `{dest}/{archive_file.file_stem()}`.
    #[builder(default = false)]
    pub use_archive_name_dir: bool,
    /// The format detector to use for determining archive format.
    #[builder(default = default_format_detector())]
    pub format_detector: Box<dyn FormatDetector>,
}

impl ExtractConfig {
    /// Determines the destination path for extraction based on configuration.
    ///
    /// This internal method calculates the final destination path,
    /// taking into account the `use_archive_name_dir` flag.
    pub(crate) fn dest(&self, archive_file: &Path) -> Result<PathBuf> {
        let dest = if self.use_archive_name_dir {
            let stem = archive_file
                .file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("archive"));
            self.dest.join(stem)
        } else {
            self.dest.clone()
        };
        if dest.exists() && !self.overwrite {
            // Extracting into the current or parent directory is the common
            // case, so their mere existence is not treated as a conflict.
            if dest == Path::new(".") || dest == Path::new("..") {
                Ok(dest)
            } else {
                Err(Error::DirExists(dest))
            }
        } else {
            Ok(dest)
        }
    }

    /// Creates an extractor for the given archive file.
    ///
    /// # Arguments
    ///
    /// * `archive_file` - Path to the archive file
    ///
    /// # Returns
    ///
    /// Returns a boxed [`ToteExtractor`](crate::extractor::ToteExtractor) for the detected format.
    pub fn extractor(
        &self,
        archive_file: &Path,
    ) -> Result<Box<dyn crate::extractor::ToteExtractor>> {
        let format = self.format_detector.detect(archive_file);
        crate::extractor::create_with(archive_file, format)
    }
}

/// Returns the entries (file list) in the given archive file.
///
/// # Arguments
///
/// * `archive_file` - The path to the archive file
/// * `format_detector` - The format detector to determine archive type
///
/// # Returns
///
/// Returns a [`Result`] containing [`Entries`] which holds the list of files in the archive.
///
/// # Examples
///
/// ```
/// use totebag::{entries, format::default_format_detector};
///
/// let detector = default_format_detector();
/// match entries("../testdata/test.zip", detector.as_ref()) {
///     Ok(entries) => {
///         for entry in entries.iter() {
///             println!("{}", entry.name);
///         }
///     }
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn entries<P: AsRef<Path>>(
    archive_file: P,
    format_detector: &dyn FormatDetector,
) -> Result<Entries> {
    let archive_file = archive_file.as_ref();
    let format = format_detector.detect(archive_file);
    let extractor = crate::extractor::create_with(archive_file, format)?;
    extractor.list(archive_file.to_path_buf())
}

/// Returns a formatted string representation of the entries in the given archive file.
///
/// # Arguments
///
/// * `archive_file` - The path to the archive file
/// * `config` - The list configuration including output format
///
/// # Returns
///
/// Returns a [`Result`] containing a formatted string of the archive entries.
/// The format depends on the [`OutputFormat`] specified in the config.
///
/// # Examples
///
/// ```
/// use totebag::{list, ListConfig, OutputFormat, format::default_format_detector};
///
/// let config = ListConfig::new(OutputFormat::Long, default_format_detector());
/// match list("../testdata/test.zip", &config) {
///     Ok(output) => println!("{}", output),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn list<P: AsRef<Path>>(archive_file: P, config: &ListConfig) -> Result<String> {
    match entries(archive_file, config.format_detector.as_ref()) {
        Err(e) => Err(e),
        Ok(entries) => format_for_output(entries, &config.format),
    }
}

fn format_for_output(entries: Entries, f: &OutputFormat) -> Result<String> {
    use OutputFormat::*;
    match f {
        Default => outputs::to_string(&entries),
        Long => outputs::to_string_long(&entries),
        Json => serde_json::to_string(&entries).map_err(Error::Json),
        PrettyJson => serde_json::to_string_pretty(&entries).map_err(Error::Json),
        Xml => serde_xml_rs::to_string(&entries).map_err(Error::Xml),
    }
}

/// Configuration for listing archive file contents.
///
/// This struct holds the options for displaying archive entries.
///
/// # Examples
///
/// ```
/// use totebag::{ListConfig, OutputFormat, format::default_format_detector};
///
/// let config = ListConfig::new(OutputFormat::Json, default_format_detector());
/// ```
pub struct ListConfig {
    /// Specify the output format for listing.
    pub format: OutputFormat,
    format_detector: Box<dyn FormatDetector>,
}

impl ListConfig {
    pub fn new(format: OutputFormat, format_detector: Box<dyn FormatDetector>) -> Self {
        Self {
            format,
            format_detector,
        }
    }
}

/// Output format options for listing archive contents.
///
/// # Variants
///
/// * `Default` - Simple list of file names, one per line
/// * `Long` - Detailed format with permissions, sizes, and dates
/// * `Json` - Compact JSON format
/// * `PrettyJson` - Human-readable JSON format with indentation
/// * `Xml` - XML format
#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum OutputFormat {
    Default,
    Long,
    Json,
    PrettyJson,
    Xml,
}

/// Create an archive file from the specified targets.
///
/// # Arguments
///
/// * `archive_targets` - A slice of paths to files or directories to archive
/// * `config` - The archive configuration
///
/// # Returns
///
/// Returns a [`Result`] containing [`ArchiveEntries`] which holds details about the created archive.
///
/// # Examples
/// ```
/// use totebag::{archive, ArchiveConfig};
/// use std::path::PathBuf;
/// let config = ArchiveConfig::builder()
///     .dest("results/doc_archive.tar.gz")  // Destination archive file and its format (by file extension).
///     .level(9)               // Maximum compression level
///     .overwrite(true)        // set overwrite flag of the destination file.
///     // .no_recursive(false) // Default is false.
///     .build();
/// let targets = vec!["src", "Cargo.toml"].iter() // files to be archived.
///    .map(|s| PathBuf::from(s)).collect::<Vec<PathBuf>>();
/// archive(&targets, &config)
///     .expect("Archiving should succeed");
/// ```
pub fn archive<P: AsRef<Path>>(
    archive_targets: &[P],
    config: &ArchiveConfig,
) -> Result<ArchiveEntries> {
    let dest_file = config.dest_file()?;
    log::info!("{:?}: {}", dest_file, dest_file.exists());
    let archiver = archiver::create(&dest_file)?;
    if let Some(parent) = dest_file.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(Error::IO)?;
    }
    let targets = prepare_targets(archive_targets);
    match std::fs::File::create(&dest_file) {
        Ok(file) => match archiver.perform(file, &targets, config) {
            Ok(entries) => {
                let compressed = dest_file.metadata().map(|m| m.len()).unwrap_or(0);
                Ok(ArchiveEntries::new(dest_file, entries, compressed))
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(Error::IO(e)),
    }
}

fn prepare_targets<P: AsRef<Path>>(targets: &[P]) -> Vec<PathBuf> {
    targets.iter().map(|p| p.as_ref().to_path_buf()).collect()
}

/// Configuration for creating archive files.
///
/// This struct holds all the options needed to create an archive file.
/// Use the builder pattern to create an instance.
///
/// # Examples
///
/// ```
/// use totebag::{ArchiveConfig, IgnoreType};
/// use std::path::PathBuf;
///
/// let config = ArchiveConfig::builder()
///     .dest("results/doc_archive.tar.gz")
///     .level(9)  // Maximum compression
///     .rebase_dir(PathBuf::from("root"))
///     .overwrite(true)
///     .no_recursive(false)
///     .ignore(vec![IgnoreType::GitIgnore, IgnoreType::Hidden])
///     .build();
/// ```
#[derive(TypedBuilder, Debug, Clone)]
pub struct ArchiveConfig {
    /// The destination file for archiving.
    #[builder(setter(into), default = PathBuf::from("totebag.zip"))]
    pub dest: PathBuf,

    /// The compression level (available: 0 to 9, 0 is none and 9 is finest).
    #[builder(default = 5)]
    pub level: u8,

    /// The directory prefixed to every entry name inside the archive.
    ///
    /// With `rebase_dir` set to `root`, the file `src/main.rs` is stored as
    /// `root/src/main.rs`. `None` (the default) stores entries under their own
    /// paths. Use [`ArchiveConfigBuilder::rebase_dir_opt`] to pass an `Option`
    /// directly.
    #[builder(default = None, setter(strip_option(fallback = rebase_dir_opt), into))]
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
    pub ignore: Vec<IgnoreType>,
}

impl ArchiveConfig {
    /// Validates and returns the destination file path.
    ///
    /// # Returns
    ///
    /// Returns the destination path if valid, or an error if:
    /// - The file exists and overwrite is false
    /// - The path is a directory
    pub fn dest_file(&self) -> Result<PathBuf> {
        let dest_path = self.dest.clone();
        if dest_path.exists() {
            if dest_path.is_file() && !self.overwrite {
                Err(Error::FileExists(dest_path))
            } else if self.dest.is_dir() {
                Err(Error::DestIsDir(dest_path))
            } else {
                Ok(dest_path)
            }
        } else {
            Ok(dest_path)
        }
    }

    /// Transforms the given path to its representation inside the archive.
    ///
    /// The path is normalized first (see [`normalize_entry_path`]), then
    /// prefixed with `rebase_dir` when one is set.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to transform
    ///
    /// # Returns
    ///
    /// The path as it should appear in the archive. It never starts at the
    /// filesystem root and never contains a `.` or `..` component.
    ///
    /// The result is empty only when nothing survives normalization, which can
    /// happen for a directory entry such as `..` itself. Callers skip those.
    pub fn path_in_archive<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let from_path = path.as_ref();
        let normalized = normalize_entry_path(from_path);
        let to_path = match &self.rebase_dir {
            Some(rebase) => normalize_entry_path(rebase).join(&normalized),
            None => normalized,
        };
        log::debug!("dest_path({from_path:?}) -> {to_path:?}");
        to_path
    }

    /// Creates an iterator over directory entries for the given path.
    ///
    /// The iterator respects the ignore settings configured in this config.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to iterate over
    ///
    /// # Returns
    ///
    /// An iterator over directory entries.
    pub fn iter<P: AsRef<Path>>(&self, path: P) -> impl Iterator<Item = ignore::DirEntry> {
        let mut builder = WalkBuilder::new(path);
        build_walker_impl(self, &mut builder);
        builder.build().flatten()
    }

    /// Returns the list of ignore types to use.
    ///
    /// If the ignore list is empty, returns a default set.
    /// The `Default` ignore type expands to multiple individual types.
    ///
    /// # Returns
    ///
    /// A vector of ignore types.
    pub fn ignore_types(&self) -> Vec<IgnoreType> {
        if self.ignore.is_empty() {
            vec![
                IgnoreType::Ignore,
                IgnoreType::GitIgnore,
                IgnoreType::GitGlobal,
                IgnoreType::GitExclude,
            ]
        } else {
            let mut r = HashSet::<IgnoreType>::new();
            for &it in &self.ignore {
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

/// Rewrites a path into a name that is safe to store in an archive.
///
/// An entry name that starts at the filesystem root, or that climbs out of the
/// archive with `..`, is a path-traversal hazard for whoever extracts it later.
/// The tar backend refuses such names outright, while zip, 7z, cab and cpio used
/// to write them verbatim (issue #90). Normalizing here keeps every backend
/// consistent and every archive totebag produces safe to extract.
///
/// The rules follow `tar(1)`, which reports "Removing leading `../' from member
/// names" for the same inputs:
///
/// - the root and any drive prefix are dropped, so `/etc/hosts` becomes `etc/hosts`;
/// - `.` components are dropped, so `./src/main.rs` becomes `src/main.rs`;
/// - `..` cancels the preceding component when there is one, so `a/../b` becomes `b`;
/// - a leading `..` that cannot cancel anything is dropped, so `../foo/bar`
///   becomes `foo/bar`.
///
/// This is a purely lexical transformation: the filesystem is never consulted, so
/// symbolic links are left for the caller to deal with.
///
/// ```
/// use std::path::{Path, PathBuf};
/// use totebag::normalize_entry_path;
///
/// assert_eq!(normalize_entry_path(Path::new("../foo/bar")), PathBuf::from("foo/bar"));
/// assert_eq!(normalize_entry_path(Path::new("./src/main.rs")), PathBuf::from("src/main.rs"));
/// assert_eq!(normalize_entry_path(Path::new("a/../b")), PathBuf::from("b"));
/// ```
pub fn normalize_entry_path<P: AsRef<Path>>(path: P) -> PathBuf {
    use std::path::Component;

    let mut components: Vec<std::ffi::OsString> = Vec::new();
    for component in path.as_ref().components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(c) => components.push(c.to_os_string()),
        }
    }
    components.iter().collect()
}

fn build_walker_impl(opts: &ArchiveConfig, w: &mut WalkBuilder) {
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_error_message() {
        use crate::Error;
        use std::path::PathBuf;

        assert_eq!(
            Error::Archiver("hoge".into()).to_string(),
            "Archiver error: hoge"
        );
        assert_eq!(
            Error::Extractor("hoge".into()).to_string(),
            "Extractor error: hoge"
        );
        assert_eq!(
            Error::DestIsDir(PathBuf::from("hoge")).to_string(),
            "hoge: Destination is a directory"
        );
        assert_eq!(
            Error::DirExists(PathBuf::from("hoge")).to_string(),
            "hoge: Directory already exists"
        );
        assert_eq!(
            Error::Fatal(Box::new(std::io::Error::other("hoge"))).to_string(),
            "Error: hoge"
        );
        assert_eq!(
            Error::Json(serde::de::Error::custom("hoge")).to_string(),
            "Json error: hoge"
        );
        assert_eq!(
            Error::Xml(serde_xml_rs::Error::Custom("hoge".into())).to_string(),
            "Xml error: Custom: hoge"
        );
        assert_eq!(
            Error::IO(std::io::Error::new(std::io::ErrorKind::NotFound, "hoge")).to_string(),
            "IO error: hoge"
        );
        assert_eq!(
            Error::FileNotFound("hoge".into()).to_string(),
            "hoge: File not found"
        );
        assert_eq!(
            Error::FileExists("hoge".into()).to_string(),
            "hoge: File already exists"
        );
        assert_eq!(
            Error::UnknownFormat("hoge".to_string()).to_string(),
            "hoge: Unknown format"
        );
        assert_eq!(
            Error::UnsupportedFormat("hoge".to_string()).to_string(),
            "hoge: Unsupported format"
        );
        assert_eq!(
            Error::unsupported_for_archiving("Rar").to_string(),
            "Rar (archiving): Unsupported format"
        );
        assert_eq!(
            Error::FeatureDisabled {
                format: "Rar".to_string(),
                feature: "rar",
            }
            .to_string(),
            "Rar: support is not compiled in (rebuild with --features rar)"
        );
        assert_eq!(
            Error::Warn("message".to_string()).to_string(),
            "Unknown error: message"
        );
        assert_eq!(
            Error::NoArgumentsGiven.to_string(),
            "No arguments given. Use --help for usage."
        );
        assert_eq!(
            Error::Array(vec![
                Error::Warn("hoge1".to_string()),
                Error::Warn("hoge2".to_string())
            ])
            .to_string(),
            "Unknown error: hoge1\nUnknown error: hoge2"
        );
    }
}
