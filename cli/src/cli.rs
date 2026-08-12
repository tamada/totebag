use clap::{Parser, ValueEnum};
use std::{io::BufRead, path::PathBuf};
use totebag::format::default_format_detector;

use totebag::{ArchiveConfig, ExtractConfig, ListConfig};
use totebag::{Error, IgnoreType, OutputFormat, Result};

pub(crate) enum Mode {
    Archive(ArchiveConfig),
    Extract(ExtractConfig),
    List(ListConfig),
}

impl Mode {
    #[cfg(test)]
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

    #[clap(
        short = 'F',
        long,
        value_name = "ARCHIVE_FORMAT",
        value_enum,
        ignore_case = true,
        help = "Specify the archive format for listing mode (default auto). available on list and extract modes."
    )]
    pub from: Option<ArchiveFormat>,

    #[cfg(feature = "completion")]
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
'-' reads from stdin, and '@<filename>' reads from a file.
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
        long = "rebase-dir",
        visible_alias = "dir",
        value_name = "DIR",
        required = false,
        help = r#"Prefix every entry in the archive with DIR (archive mode).
For example, -C root stores src/main.rs as root/src/main.rs.
By default entries keep their own paths."#
    )]
    pub rebase_dir: Option<PathBuf>,

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

/// The archive format that `--from` may name.
///
/// The variants after `Parse` are resolved through
/// [`totebag::format::find_format`], which accepts both canonical names
/// (`TarGz`) and extension aliases (`tgz`, `jar`, `lzh`).
#[derive(Parser, Debug, ValueEnum, Clone, PartialEq, Copy)]
pub enum ArchiveFormat {
    /// Detect the format by the file extension.
    Auto,
    /// Detect the format by the file signature (header bytes).
    Parse,
    Ar,
    Cab,
    Cpio,
    Lha,
    Lzh,
    SevenZ,
    Rar,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZstd,
    Zip,
    Tgz,
    Tbz2,
    Txz,
    Tzst,
    Tzstd,
    Jar,
    War,
    Ear,
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
            Err(Error::NoArgumentsGiven)
        } else {
            match self.mode {
                RunMode::Auto => {
                    let fd = default_format_detector();
                    if totebag::format::is_all_archive_file(&args, fd.as_ref()) {
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
        use totebag::format::{
            default_format_detector, fixed_format_detector, magic_number_format_detector,
        };
        match self.from {
            Some(ArchiveFormat::Auto) | None => Ok(default_format_detector()),
            Some(ArchiveFormat::Parse) => Ok(magic_number_format_detector()),
            Some(f) => {
                let name = format!("{f:?}");
                let format = totebag::format::find_format(&name)
                    .ok_or_else(|| Error::UnsupportedFormat(name))?;
                Ok(fixed_format_detector(format))
            }
        }
    }
}

/// Determines the destination of an archive, following the rule documented in
/// `--help`:
///
/// 1. if the `--output` option is given, use it;
/// 2. otherwise, if the first argument names an archive file, use it and drop it
///    from the list of targets;
/// 3. otherwise, fall back to `totebag.zip`.
fn to_archive_config(opts: &CliOpts, args: Vec<String>) -> Result<(Mode, Vec<String>)> {
    let fd = default_format_detector();
    let (dest, args) = match &opts.output {
        Some(output) => (output.clone(), args),
        None if fd.detect(&PathBuf::from(&args[0])).is_some() => {
            (PathBuf::from(&args[0]), args[1..].to_vec())
        }
        None => (PathBuf::from("totebag.zip"), args),
    };
    let config = totebag::ArchiveConfig::builder()
        .dest(dest)
        .level(opts.archivers.level)
        .rebase_dir_opt(opts.archivers.rebase_dir.clone())
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
    let config =
        totebag::ListConfig::new(opts.listers.output_format.clone(), opts.format_detector()?);
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
            .collect::<Vec<Error>>();
        Err(Error::Array(errs))
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
        Err(Error::FileNotFound(path))
    } else {
        match std::fs::File::open(path) {
            Ok(f) => reads_from_reader(f),
            Err(e) => Err(Error::IO(e)),
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
        let cli = CliOpts::parse_from(["totebag_test", "@../testdata/files/archive_mode1.txt"]);
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
        let cli = CliOpts::parse_from(["totebag_test", "@../testdata/files/archive_mode2.txt"]);
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

    /// `--output` takes precedence over an archive name in the first argument,
    /// as `--help` documents. Regression test for the bug where `-o` was dropped
    /// and everything landed in `totebag.zip` (issue #86).
    #[test]
    fn test_output_option_wins_over_first_argument() {
        let cli = CliOpts::parse_from([
            "totebag_test",
            "-m",
            "archive",
            "-o",
            "explicit.tar.gz",
            "src",
            "LICENSE",
        ]);
        let (mode, args) = cli.find_mode().unwrap();
        match mode {
            Mode::Archive(config) => {
                assert_eq!(config.dest, PathBuf::from("explicit.tar.gz"));
            }
            _ => panic!("invalid mode"),
        }
        // the targets are untouched when -o supplies the destination
        assert_eq!(args, vec!["src", "LICENSE"]);
    }

    /// Without `--output`, a leading archive name still becomes the destination
    /// and is dropped from the targets.
    #[test]
    fn test_first_argument_is_used_as_destination() {
        let cli = CliOpts::parse_from(["totebag_test", "-m", "archive", "out.zip", "src"]);
        let (mode, args) = cli.find_mode().unwrap();
        match mode {
            Mode::Archive(config) => assert_eq!(config.dest, PathBuf::from("out.zip")),
            _ => panic!("invalid mode"),
        }
        assert_eq!(args, vec!["src"]);
    }

    /// `-C` is the rebase prefix, and is absent unless asked for. It used to
    /// default to `.`, which put a `./` in front of every entry name.
    #[test]
    fn test_rebase_dir_is_absent_by_default() {
        let cli = CliOpts::parse_from(["totebag_test", "-m", "archive", "out.zip", "src"]);
        let (mode, _) = cli.find_mode().unwrap();
        match mode {
            Mode::Archive(config) => {
                assert_eq!(config.rebase_dir, None);
                assert_eq!(
                    config.path_in_archive("src/main.rs"),
                    PathBuf::from("src/main.rs")
                );
            }
            _ => panic!("invalid mode"),
        }
    }

    #[test]
    fn test_rebase_dir_prefixes_entries() {
        for flag in ["-C", "--rebase-dir", "--dir"] {
            let cli = CliOpts::parse_from([
                "totebag_test",
                "-m",
                "archive",
                flag,
                "root",
                "out.zip",
                "src",
            ]);
            let (mode, _) = cli.find_mode().unwrap();
            match mode {
                Mode::Archive(config) => assert_eq!(
                    config.path_in_archive("src/main.rs"),
                    PathBuf::from("root/src/main.rs"),
                    "for {flag}"
                ),
                _ => panic!("invalid mode"),
            }
        }
    }

    /// Every `--from` value must build a detector rather than failing with
    /// "Unsupported format" (issue #73, issue #74).
    #[test]
    fn test_every_from_value_builds_a_detector() {
        for format in ArchiveFormat::value_variants() {
            let cli = CliOpts::parse_from(["totebag_test", "-m", "list", "dummy.zip"]);
            let cli = CliOpts {
                from: Some(*format),
                ..cli
            };
            assert!(
                cli.format_detector().is_ok(),
                "--from {format:?} should be resolvable"
            );
        }
    }

    #[test]
    fn test_read_from_file3() {
        let cli = CliOpts::parse_from(["totebag_test", "@../testdata/files/extract_mode.txt"]);
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
            CliOpts::parse_from(["totebag_test", "src", "LICENSE", "README.md", "Cargo.toml"]);
        let (mode, args) = cli1.find_mode().unwrap();
        assert_eq!(mode.mode(), "archive");
        assert_eq!(args, vec!["src", "LICENSE", "README.md", "Cargo.toml"]);
    }

    #[test]
    fn test_find_mode_2() {
        let cli2 = CliOpts::parse_from(["totebag_test", "src", "LICENSE", "README.md", "hoge.zip"]);
        let (mode, args) = cli2.find_mode().unwrap();
        assert_eq!(mode.mode(), "archive");
        assert_eq!(args, vec!["src", "LICENSE", "README.md", "hoge.zip"]);
    }

    #[test]
    fn test_find_mode_3() {
        let cli3 = CliOpts::parse_from([
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
        let cli4 = CliOpts::parse_from([
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
        let r = CliOpts::try_parse_from(["totebag_test"]);
        assert!(r.is_err());
    }
}
