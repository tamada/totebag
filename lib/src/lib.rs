//! # Totebag
//!
//! `totebag` is an archiving utilities that can archive and extract files supported several formats.
//!
pub mod archiver;
pub mod extractor;
pub mod format;

use clap::ValueEnum;
use std::path::PathBuf;

/// Define the result type for the this library.
pub type Result<T> = std::result::Result<T, ToteError>;

/// Define the ignore types for directory traversing.
#[derive(Debug, Clone, ValueEnum, PartialEq, Copy, Hash, Eq)]
pub enum IgnoreType {
    /// [IgnoreType::GitIgnore], [IgnoreType::GitGlobal], [IgnoreType::GitExclude], and [IgnoreType::Ignore].
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
    NoArgumentsGiven,
    Warn(String),
    UnknownFormat(String),
    UnsupportedFormat(String),
}
