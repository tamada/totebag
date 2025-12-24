use clap::{Parser, ValueEnum};
use totebag::format::FixedFormatDetector;
use std::{io::BufRead, path::PathBuf};

use totebag::{ArchiveConfig, ExtractConfig, ListConfig};
use totebag::{IgnoreType, OutputFormat, Result, ToteError};

pub(crate) enum Mode {
    Archive(ArchiveConfig),
    Extract(ExtractConfig),
    List(ListConfig),
}

impl Mode {
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub(crate) fn mode(&self) -> String {
        match self {
            Self::Archive(_) => "archive",
            Self::Extract(_) => "extract",
            Self::List(_) => "list",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub(crate) enum RunMode {
    Auto,
    Archive,
    Extract,
    List,
}

#[derive(Parser, Debug)]
#[clap(
    bin_name = "totebag",
    version,
    author,
    about,
    arg_required_else_help = true
)]
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

    #[clap(short = 'F', long, value_name = "ARCHIVE_FORMAT", value_enum, ignore_case = true,
        help = "Specify the archive format for listing mode (default auto). available on list and extract modes.")]
    pub from: Option<ArchiveFormat>,

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
    pub args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ListerOpts {
    #[clap(
        short = 'f', long, value_name = "FORMAT", value_enum, ignore_case = true,
        default_value_t = OutputFormat::Default,
        help = "Specify the format for listing entries in the archive file."
    )]
    pub output_format: OutputFormat,
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

#[derive(Parser, Debug, ValueEnum, Clone, PartialEq, Copy)]
pub enum ArchiveFormat {
    /// Detect the format by the file extension.
    Auto,
    /// Detect the format by the file signature (header bytes).
    Parse,
    Cab, Lha, Lzh, SevenZ, Rar, Tar, TarGz, TarBz2, TarXz, TarZstd, Zip,
    Tgz, Tbz2, Txz, Tzst, Tzstd, Jar, War, Ear,
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
    pub(crate) fn find_mode(&self) -> Result<(Mode, Vec<String>)> {
        let args = normalize_args(self.args.clone())?;
        if args.is_empty() {
            Err(ToteError::NoArgumentsGiven)
        } else {
            match self.mode {
                RunMode::Auto => {
                    if totebag::format::match_all(&args) {
                        to_extract_config(self, args)
                    } else {
                        to_archive_config(self, args)
                    }
                }
                RunMode::Archive => to_archive_config(self, args),
                RunMode::Extract => to_extract_config(self, args),
                RunMode::List => to_list_config(self, args),
            }
        }
    }

    fn format_detector(&self) -> Result<Box<dyn totebag::format::FormatDetector>> {
        use totebag::format::{ExtensionFormatDetector, MagicNumberFormatDetector};
        match self.from {
            Some(ArchiveFormat::Auto) | None => {
                Ok(Box::new(ExtensionFormatDetector {}))
            }
            Some(ArchiveFormat::Parse) => Ok(Box::new(MagicNumberFormatDetector {})),
            Some(f) => {
                let name = format!("{f:?}");
                let format = totebag::format::find_by_name(name).ok_or_else(|| {
                    ToteError::UnsupportedFormat(format!(
                        "The specified archive format '{f:?}' is not supported."
                    ))
                })?;
                Ok(Box::new(FixedFormatDetector::new(format)))
            }
        }
    }
}


fn to_archive_config(opts: &CliOpts, args: Vec<String>) -> Result<(Mode, Vec<String>)> {
    let (dest, args) = if totebag::format::find(&args[0]).is_some() && opts.output.is_none() {
        (Some(args[0].clone().into()), args[1..].to_vec())
    } else {
        (None, args)
    };
    let config = totebag::ArchiveConfig::builder()
        .dest(dest.unwrap_or_else(|| PathBuf::from("totebag.zip")))
        .level(opts.archivers.level)
        .rebase_dir(opts.archivers.base_dir.clone())
        .overwrite(opts.overwrite)
        .no_recursive(opts.archivers.no_recursive)
        .ignore(opts.archivers.ignores.clone())
        .build();
    Ok((Mode::Archive(config), args))
}

fn to_extract_config(opts: &CliOpts, args: Vec<String>) -> Result<(Mode, Vec<String>)> {
    let dest = opts.output.clone().unwrap_or_else(|| PathBuf::from("."));
    let config = totebag::ExtractConfig::builder()
        .overwrite(opts.overwrite)
        .use_archive_name_dir(opts.extractors.to_archive_name_dir)
        .dest(dest)
        .format_detector(opts.format_detector()?)
        .build();
    Ok((Mode::Extract(config), args))
}

fn to_list_config(opts: &CliOpts, args: Vec<String>) -> Result<(Mode, Vec<String>)> {
    let config = totebag::ListConfig::new(
        opts.listers.output_format.clone(),
        opts.format_detector()?,
    );
    Ok((Mode::List(config), args))
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
    let path = PathBuf::from(s.as_ref());
    if !path.exists() {
        Err(ToteError::FileNotFound(path))
    } else {
        match std::fs::File::open(path) {
            Ok(f) => reads_from_reader(f),
            Err(e) => Err(ToteError::IO(e)),
        }
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
        let cli = CliOpts::parse_from(&["totebag_test", "@../testdata/files/archive_mode1.txt"]);
        let (mode, args) = cli.find_mode().unwrap();
        match mode {
            Mode::List(_) | Mode::Extract(_) => panic!("invalid mode"),
            Mode::Archive(config) => assert_eq!(
                config.dest_file().unwrap(),
                PathBuf::from("testdata/targets.tar.gz")
            ),
        }
        assert_eq!(
            args,
            vec!["src", "README.md", "LICENSE", "Cargo.toml", "Makefile.toml"]
        );
    }

    #[test]
    fn test_read_from_file2() {
        let cli = CliOpts::parse_from(&["totebag_test", "@../testdata/files/archive_mode2.txt"]);
        let (mode, args) = cli.find_mode().unwrap();
        match mode {
            Mode::List(_) | Mode::Extract(_) => panic!("invalid mode"),
            Mode::Archive(config) => {
                assert_eq!(config.dest_file().unwrap(), PathBuf::from("totebag.zip"))
            }
        }
        assert_eq!(
            args,
            vec!["src", "README.md", "LICENSE", "Cargo.toml", "Makefile.toml"]
        );
    }

    #[test]
    fn test_read_from_file3() {
        let cli = CliOpts::parse_from(&["totebag_test", "@../testdata/files/extract_mode.txt"]);
        let (mode, args) = cli.find_mode().unwrap();
        match mode {
            Mode::List(_) | Mode::Archive(_) => panic!("invalid mode"),
            Mode::Extract(config) => assert_eq!(config.dest, PathBuf::from(".")),
        }
        assert_eq!(args, vec!["testdata/test.cab", "testdata/test.tar"]);
    }

    #[test]
    fn test_find_mode_1() {
        let cli1 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "Cargo.toml"]);
        let (mode, args) = cli1.find_mode().unwrap();
        assert_eq!(mode.mode(), "archive");
        assert_eq!(args, vec!["src", "LICENSE", "README.md", "Cargo.toml"]);
    }

    #[test]
    fn test_find_mode_2() {
        let cli2 =
            CliOpts::parse_from(&["totebag_test", "src", "LICENSE", "README.md", "hoge.zip"]);
        let (mode, args) = cli2.find_mode().unwrap();
        assert_eq!(mode.mode(), "archive");
        assert_eq!(args, vec!["src", "LICENSE", "README.md", "hoge.zip"]);
    }

    #[test]
    fn test_find_mode_3() {
        let cli3 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
        ]);
        let (mode, args) = cli3.find_mode().unwrap();
        assert_eq!(mode.mode(), "extract");
        assert_eq!(
            args,
            vec!["src.zip", "LICENSE.tar", "README.tar.bz2", "hoge.rar"]
        );
    }

    #[test]
    fn test_find_mode_4() {
        let cli4 = CliOpts::parse_from(&[
            "totebag_test",
            "src.zip",
            "LICENSE.tar",
            "README.tar.bz2",
            "hoge.rar",
            "--mode",
            "list",
        ]);
        let (mode, args) = cli4.find_mode().unwrap();
        assert_eq!(mode.mode(), "list");
        assert_eq!(
            args,
            vec!["src.zip", "LICENSE.tar", "README.tar.bz2", "hoge.rar"]
        );
    }

    #[test]
    fn test_cli_parse_error() {
        let r = CliOpts::try_parse_from(&["totebag_test"]);
        assert!(r.is_err());
    }
}
