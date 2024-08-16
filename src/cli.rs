use std::path::PathBuf;
use clap::{Parser, ValueEnum};

pub type Result<T> = std::result::Result<T, ToteError>;

#[derive(Parser, Debug)]
#[clap(
    version, author, about,
    arg_required_else_help = true,
)]
pub struct CliOpts {
    #[clap(short = 'm', long = "mode", default_value_t = RunMode::Auto, value_name = "MODE", required = false, ignore_case = true, value_enum, help = "Mode of operation.")]
    pub mode: RunMode,
    #[clap(short = 'o', short_alias = 'd', long = "output", alias = "dest", value_name = "DEST", required = false, help = "Output file in archive mode, or output directory in extraction mode")]
    pub output: Option<PathBuf>,
    #[clap(long = "to-archive-name-dir", help = "extract files to DEST/ARCHIVE_NAME directory (extract mode).", default_value_t = false)]
    pub to_archive_name_dir: bool,

    #[clap(
        short = 'C', long = "dir", value_name = "DIR", required = false,
        default_value = ".", help = "Specify the base directory for archiving or extracting."
    )]
    pub base_dir: PathBuf,

    #[clap(short = 'n', long = "no-recursive", help = "No recursive directory (archive mode).", default_value_t = false)]
    pub no_recursive: bool,
    #[clap(short = 'v', long = "verbose", help = "Display verbose output.", default_value_t = false)]
    pub verbose: bool,
    #[clap(long, help = "Overwrite existing files.")]
    pub overwrite: bool,
    #[clap(value_name = "ARGUMENTS", help = r###"List of files or directories to be processed.
If archive mode, the archive file name can specify at the first argument.
If the frist argument was not the archive name, the default archive name `totebag.zip` is applied.
"###)]
    pub args: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub enum RunMode {
    Auto,
    Archive,
    Extract,
    List,
}

#[derive(Debug)]
pub enum ToteError {
    NoArgumentsGiven,
    FileNotFound(PathBuf),
    FileExists(PathBuf),
    IO(std::io::Error),
    Archiver(String),
    UnsupportedFormat(String),
    UnknownFormat(String),
    Unknown(String),
    Fatal(Box<dyn std::error::Error>)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_find_mode() {
        let mut cli1 = CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "Cargo.toml"]);
        let r1 = cli1.run_mode();
        assert!(r1.is_ok());
        assert_eq!(r1.unwrap(), RunMode::Archive);

        let mut cli2 = CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "hoge.zip"]);
        let r2 = cli2.run_mode();
        assert!(r2.is_ok());
        assert_eq!(cli2.run_mode().unwrap(), RunMode::Archive);

        let mut cli3 = CliOpts::parse_from(&["totebag_test", "src.zip", "LICENSE.tar", "README.tar.bz2", "hoge.rar"]);
        let r3 = cli3.run_mode();
        assert!(r3.is_ok());
        assert_eq!(cli3.run_mode().unwrap(), RunMode::Extract);

        let mut cli4 = CliOpts::parse_from(&["totebag_test", "src.zip", "LICENSE.tar", "README.tar.bz2", "hoge.rar", "--mode", "list"]);
        let r4 = cli3.run_mode();
        assert!(r4.is_ok());
        assert_eq!(cli4.run_mode().unwrap(), RunMode::List);
    }
}
