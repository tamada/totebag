use clap::{Parser, ValueEnum};
use std::{io::BufRead, path::PathBuf};

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

    #[clap(long = "log", help = "Specify the log level", default_value_t = LogLevel::Warn, ignore_case = true, value_enum)]
    pub loglevel: LogLevel,

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
'-' reads form stdin, and '@<filename>' reads from a file.
In archive mode, the resultant archive file name is determined by the following rule.
    - if output option is specified, use it.
    - if the first argument is the archive file name, use it.
    - otherwise, use the default name 'totebag.zip'.
The format is determined by the extension of the resultant file name."###
    )]
    args: Vec<String>,
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

    #[clap(short = 'L', long = "level", default_value_t = 5, help = r#"Specify the compression level. [default: 5] [possible values: 0-9 (none to finest)]
For more details of level of each compression method, see README."#, value_parser=compression_level)]
    pub level: u8,

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

fn compression_level(s: &str) -> core::result::Result<u8, String> {
    clap_num::number_range(s, 0, 9)
}

#[derive(Parser, Debug)]
struct ActualArgs {
    args: Vec<String>,
}

impl ActualArgs {}

impl CliOpts {
    pub fn args(&self) -> Vec<String> {
        self.args.clone()
    }

    pub fn run_mode(&self) -> RunMode {
        self.mode
    }

    pub fn archiver_output(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from("totebag.zip"))
    }

    pub fn extractor_output(&self) -> PathBuf {
        self.output.clone().unwrap_or_else(|| PathBuf::from("."))
    }

    pub(crate) fn finalize(&mut self, m: &totebag::format::Manager) -> Result<()> {
        let args = match normalize_args(self.args.clone()) {
            Ok(args) => args,
            Err(e) => return Err(e),
        };
        if args.is_empty() {
            return Err(ToteError::NoArgumentsGiven);
        }
        if self.mode == RunMode::Auto {
            if m.match_all(&args) {
                self.args = args;
                self.mode = RunMode::Extract;
            } else {
                self.mode = RunMode::Archive;
                if m.find(&args[0]).is_some() && self.output.is_none() {
                    self.output = Some(args[0].clone().into());
                    self.args = args[1..].to_vec();
                } else {
                    self.args = args;
                }
            }
        } else {
            self.args = args;
        }
        Ok(())
    }
}

pub(crate) fn normalize_args(args: Vec<String>) -> Result<Vec<String>> {
    let results = args
        .iter()
        .map(reads_file_or_stdin_if_needed)
        .collect::<Vec<Result<Vec<String>>>>();
    if results.iter().any(|r| r.is_err()) {
        let errs = results
            .into_iter()
            .filter(|r| r.is_err())
            .flat_map(|r| r.err())
            .collect::<Vec<ToteError>>();
        Err(ToteError::Array(errs))
    } else {
        let results = results
            .into_iter()
            .filter(|r| r.is_ok())
            .flat_map(|r| r.unwrap())
            .collect::<Vec<String>>();
        Ok(results)
    }
}

fn reads_file_or_stdin_if_needed<S: AsRef<str>>(s: S) -> Result<Vec<String>> {
    let s = s.as_ref();
    if s == "-" {
        reads_from_reader(std::io::stdin())
    } else if let Some(stripped_str) = s.strip_prefix('@') {
        reads_from_file(stripped_str)
    } else {
        Ok(vec![s.to_string()])
    }
}

fn reads_from_file<S: AsRef<str>>(s: S) -> Result<Vec<String>> {
    match std::fs::File::open(s.as_ref()) {
        Ok(f) => reads_from_reader(f),
        Err(e) => Err(ToteError::IO(e)),
    }
}

fn reads_from_reader<R: std::io::Read>(r: R) -> Result<Vec<String>> {
    let results = std::io::BufReader::new(r)
        .lines()
        .map_while(|r| r.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<String>>();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_read_from_file1() {
        let manager = totebag::format::Manager::default();
        let mut cli = CliOpts::parse_from(&["totebag_test", "@testdata/files/archive_mode1.txt"]);
        match cli.finalize(&manager) {
            Ok(_) => {}
            Err(e) => panic!("error: {:?}", e),
        }
        assert_eq!(cli.run_mode(), RunMode::Archive);
        assert_eq!(
            cli.args(),
            vec!["src", "README.md", "LICENSE", "Cargo.toml", "Makefile.toml"]
        );
        assert_eq!(cli.output, Some(PathBuf::from("testdata/targets.tar.gz")));
    }

    #[test]
    fn test_read_from_file2() {
        let manager = totebag::format::Manager::default();
        let mut cli = CliOpts::parse_from(&["totebag_test", "@testdata/files/archive_mode2.txt"]);
        match cli.finalize(&manager) {
            Ok(_) => {}
            Err(e) => panic!("error: {:?}", e),
        }
        assert_eq!(cli.run_mode(), RunMode::Archive);
        assert_eq!(
            cli.args(),
            vec!["src", "README.md", "LICENSE", "Cargo.toml", "Makefile.toml"]
        );
        assert!(cli.output.is_none());
    }

    #[test]
    fn test_read_from_file3() {
        let manager = totebag::format::Manager::default();
        let mut cli = CliOpts::parse_from(&["totebag_test", "@testdata/files/extract_mode.txt"]);
        match cli.finalize(&manager) {
            Ok(_) => {}
            Err(e) => panic!("error: {:?}", e),
        }
        assert_eq!(cli.run_mode(), RunMode::Extract);
        assert_eq!(cli.args(), vec!["testdata/test.cab", "testdata/test.tar"]);
        assert!(cli.output.is_none());
    }

    #[test]
    fn test_find_mode_1() {
        let manager = totebag::format::Manager::default();
        let mut cli1 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "Cargo.toml"]);
        assert!(cli1.finalize(&manager).is_ok());
        assert_eq!(cli1.run_mode(), RunMode::Archive);
    }

    #[test]
    fn test_find_mode_2() {
        let manager = totebag::format::Manager::default();
        let mut cli2 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "hoge.zip"]);
        assert!(cli2.finalize(&manager).is_ok());
        assert_eq!(cli2.run_mode(), RunMode::Archive);
    }

    #[test]
    fn test_find_mode_3() {
        let manager = totebag::format::Manager::default();
        let mut cli3 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
        ]);
        assert!(cli3.finalize(&manager).is_ok());
        assert_eq!(cli3.run_mode(), RunMode::Extract);
    }

    #[test]
    fn test_find_mode_4() {
        let manager = totebag::format::Manager::default();
        let mut cli4 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
            "--mode",
            "list",
        ]);
        assert!(cli4.finalize(&manager).is_ok());
        assert_eq!(cli4.run_mode(), RunMode::List);
    }

    #[test]
    fn test_cli_parse_error() {
        let r = CliOpts::try_parse_from(&["totebag_test"]);
        assert!(r.is_err());
    }
}
