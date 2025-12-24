//! # Totebag
//!
//! `totebag` is an archiving utilities that can archive and extract files supported several formats.
//!
pub mod archiver;
pub mod extractor;
pub mod format;
pub(crate) mod outputs;

use clap::ValueEnum;
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use typed_builder::TypedBuilder;

use crate::archiver::ArchiveEntries;
use crate::extractor::Entries;
use crate::format::{default_format_detector, FormatDetector};

/// Define the result type for the this library.
pub type Result<T> = std::result::Result<T, ToteError>;

/// Define the ignore types for directory traversing.
#[derive(Debug, Clone, ValueEnum, PartialEq, Copy, Hash, Eq)]
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

/// Define the errors for this library.
#[derive(Debug)]
pub enum ToteError {
    Archiver(String),
    Array(Vec<ToteError>),
    DestIsDir(PathBuf),
    DirExists(PathBuf),
    Extractor(String),
    Fatal(Box<dyn std::error::Error>),
    FileNotFound(PathBuf),
    FileExists(PathBuf),
    IO(std::io::Error),
    Json(serde_json::Error),
    NoArgumentsGiven,
    Warn(String),
    UnknownFormat(String),
    UnsupportedFormat(String),
    Xml(serde_xml_rs::Error),
}

impl ToteError {
    pub fn error_or<T>(ok: T, errs: Vec<Self>) -> Result<T> {
        if errs.is_empty() {
            Ok(ok)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(ToteError::Array(errs))
        }
    }
}

pub fn extract<P: AsRef<Path>>(archive_file: P, config: &ExtractConfig) -> Result<()> {
    let archive_file = archive_file.as_ref();
    let base_dir = config.dest(archive_file)?;
    let extractor = config.extractor(archive_file)?;
    extractor.perform(archive_file.to_path_buf(), base_dir)
}

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
    #[builder(default = default_format_detector())]
    pub format_detector: Box<dyn FormatDetector>,
}

impl ExtractConfig {
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
            if dest == PathBuf::from(".") || dest == PathBuf::from("..") {
                Ok(dest)
            } else {
                Err(ToteError::DirExists(dest))
            }
        } else {
            Ok(dest)
        }
    }

    pub fn extractor(&self, archive_file: &Path) -> Result<Box<dyn crate::extractor::ToteExtractor>> {
        let format = self.format_detector.detect(archive_file);
        crate::extractor::create_with(archive_file, format)
    }
}

/// Returns the entries in the given archive file.
pub fn entries<P: AsRef<Path>>(archive_file: P, format_detector: &Box<dyn FormatDetector>) -> Result<Entries> {
    let archive_file = archive_file.as_ref();
    let format = format_detector.detect(archive_file);
    let extractor = crate::extractor::create_with(archive_file, format)?;
    extractor.list(archive_file.to_path_buf())
}

/// Returns the string of the entries in the given archive file.
pub fn list<P: AsRef<Path>>(archive_file: P, config: &ListConfig) -> Result<String> {
    match entries(archive_file, &config.format_detector) {
        Err(e) => Err(e),
        Ok(entries) => format_for_output(entries, &config.format),
    }
}

fn format_for_output(entries: Entries, f: &OutputFormat) -> Result<String> {
    use OutputFormat::*;
    match f {
        Default => outputs::to_string(&entries),
        Long => outputs::to_string_long(&entries),
        Json => serde_json::to_string(&entries).map_err(ToteError::Json),
        PrettyJson => serde_json::to_string_pretty(&entries).map_err(ToteError::Json),
        Xml => serde_xml_rs::to_string(&entries).map_err(ToteError::Xml),
    }
}

/// The config object for List mode.
pub struct ListConfig {
    /// Specify the output format for listing.
    pub format: OutputFormat,
    format_detector: Box<dyn FormatDetector>,
}

impl ListConfig {
    pub fn new(format: OutputFormat, format_detector: Box<dyn FormatDetector>) -> Self {
        Self { format, format_detector }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    Default,
    Long,
    Json,
    PrettyJson,
    Xml,
}

pub fn archive<P: AsRef<Path>>(
    archive_targets: &[P],
    config: &ArchiveConfig,
) -> Result<ArchiveEntries> {
    let dest_file = config.dest_file()?;
    log::info!("{:?}: {}", dest_file, dest_file.exists());
    let archiver = archiver::create(&dest_file)?;
    if let Some(parent) = dest_file.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(ToteError::IO(e));
            }
        }
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
        Err(e) => Err(ToteError::IO(e)),
    }
}

fn prepare_targets<P: AsRef<Path>>(targets: &[P]) -> Vec<PathBuf> {
    targets
        .iter()
        .map(|p| p.as_ref().to_path_buf())
        .collect()
}

#[derive(TypedBuilder, Debug, Clone)]
pub struct ArchiveConfig {
    /// The destination file for archiving.
    #[builder(setter(into), default = PathBuf::from("totebag.zip"))]
    pub dest: PathBuf,

    /// The compression level (available: 0 to 9, 0 is none and 9 is finest).
    #[builder(default = 5)]
    pub level: u8,

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
    pub ignore: Vec<IgnoreType>,
}

impl ArchiveConfig {
    pub fn dest_file(&self) -> Result<PathBuf> {
        let dest_path = self.dest.clone();
        if dest_path.exists() {
            if dest_path.is_file() && !self.overwrite {
                Err(ToteError::FileExists(dest_path))
            } else if self.dest.is_dir() {
                Err(ToteError::DestIsDir(dest_path))
            } else {
                Ok(dest_path)
            }
        } else {
            Ok(dest_path)
        }
    }

    pub fn path_in_archive<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let from_path = path.as_ref();
        let to_path = if let Some(rebase) = &self.rebase_dir {
            rebase.join(from_path)
        } else {
            from_path.to_path_buf()
        };
        log::debug!("dest_path({from_path:?}) -> {to_path:?}");
        to_path
    }

    pub fn iter<P: AsRef<Path>>(&self, path: P) -> impl Iterator<Item = ignore::DirEntry> {
        let mut builder = WalkBuilder::new(path);
        build_walker_impl(self, &mut builder);
        builder.build().flatten()
    }

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
