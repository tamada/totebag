use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use totebag::{IgnoreType, Result, ToteError};

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub(crate) enum RunMode {
    Auto,
    Archive,
    Extract,
    List,
}

#[derive(Parser, Debug)]
#[clap(version, author, about, arg_required_else_help = true)]
pub(crate) struct CliOpts {
    #[clap(flatten)]
    pub extractors: ExtractorOpts,

    #[clap(flatten)]
    pub archivers: ArchiverOpts,

    #[clap(flatten)]
    pub listers: ListerOpts,

    #[clap(long = "level", help = "Specify the log level", default_value_t = LogLevel::Warn, ignore_case = true, value_enum)]
    pub level: LogLevel,

    #[clap(short = 'm', long = "mode", default_value_t = RunMode::Auto, value_name = "MODE", required = false, ignore_case = true, value_enum, help = "Mode of operation.")]
    pub mode: RunMode,

    #[cfg(debug_assertions)]
    #[clap(
        long = "generate-completion",
        hide = true,
        help = "Generate the completion files"
    )]
    pub generate_completion: bool,

    #[clap(
        short = 'o',
        short_alias = 'd',
        long = "output",
        alias = "dest",
        value_name = "DEST",
        required = false,
        help = "Output file in archive mode, or output directory in extraction mode"
    )]
    pub output: Option<PathBuf>,

    #[clap(long, help = "Overwrite existing files.")]
    pub overwrite: bool,

    #[clap(
        value_name = "ARGUMENTS",
        help = r###"List of files or directories to be processed.
If archive mode, the archive file name can specify at the first argument.
If the frist argument was not the archive name, the default archive name `totebag.zip` is applied.
"###
    )]
    pub args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ListerOpts {
    #[clap(
        short,
        long,
        help = "List entries in the archive file with long format."
    )]
    pub long: bool,
}

#[derive(Parser, Debug)]
pub struct ArchiverOpts {
    #[clap(
        short = 'C',
        long = "dir",
        value_name = "DIR",
        required = false,
        default_value = ".",
        help = "Specify the base directory for archiving or extracting."
    )]
    pub base_dir: PathBuf,

    #[clap(
        short = 'i',
        long = "ignore-types",
        value_name = "IGNORE_TYPES",
        value_delimiter = ',',
        help = "Specify the ignore type."
    )]
    pub ignores: Vec<IgnoreType>,

    #[clap(
        short = 'n',
        long = "no-recursive",
        help = "No recursive directory (archive mode).",
        default_value_t = false
    )]
    pub no_recursive: bool,
}

#[derive(Parser, Debug)]
pub struct ExtractorOpts {
    #[clap(
        long = "to-archive-name-dir",
        help = "extract files to DEST/ARCHIVE_NAME directory (extract mode).",
        default_value_t = false
    )]
    pub to_archive_name_dir: bool,
}

/// The log level.
#[derive(Parser, Debug, ValueEnum, Clone, PartialEq, Copy)]
pub enum LogLevel {
    /// The error level.
    Error,
    /// The warning level.
    Warn,
    /// The info level.
    Info,
    /// The debug level.
    Debug,
    /// The trace level.
    Trace,
}

impl CliOpts {
    /// Find the mode of operation.
    pub(crate) fn run_mode(&mut self, m: &totebag::format::Manager) -> Result<RunMode> {
        if self.args.is_empty() {
            return Err(ToteError::NoArgumentsGiven);
        }
        if self.mode == RunMode::Auto {
            if m.match_all(
                &self
                    .args
                    .iter()
                    .map(PathBuf::from)
                    .collect::<Vec<PathBuf>>(),
            ) {
                self.mode = RunMode::Extract;
                Ok(RunMode::Extract)
            } else {
                self.mode = RunMode::Archive;
                Ok(RunMode::Archive)
            }
        } else {
            Ok(self.mode)
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_find_mode() {
        let manager = totebag::format::Manager::default();
        let mut cli1 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "Cargo.toml"]);
        let r1 = cli1.run_mode(&manager);
        assert!(r1.is_ok());
        assert_eq!(r1.unwrap(), RunMode::Archive);

        let mut cli2 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "hoge.zip"]);
        let r2 = cli2.run_mode(&manager);
        assert!(r2.is_ok());
        assert_eq!(r2.unwrap(), RunMode::Archive);

        let mut cli3 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
        ]);
        let r3 = cli3.run_mode(&manager);
        assert!(r3.is_ok());
        assert_eq!(r3.unwrap(), RunMode::Extract);

        let mut cli4 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
            "--mode",
            "list",
        ]);
        let r4 = cli4.run_mode(&manager);
        assert!(r4.is_ok());
        assert_eq!(r4.unwrap(), RunMode::List);

        let r = CliOpts::try_parse_from(&["totebag_test"]);
        assert!(r.is_err());
    }
}
