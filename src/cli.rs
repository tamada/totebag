use std::path::PathBuf;
use clap::{Parser, ValueEnum};

pub type Result<T> = std::result::Result<T, ToatError>;

#[derive(Parser, Debug)]
#[clap(
    version, author, about,
    arg_required_else_help = true,
)]
pub struct CliOpts {
    #[clap(short = 'm', long = "mode", default_value_t = RunMode::Auto, value_name = "MODE", required = false, ignore_case = true, value_enum, help = "Mode of operation.")]
    pub mode: RunMode,
    #[clap(short = 'd', long = "dest", default_value = ".", value_name = "DEST", required = false, help = "Destination of the extraction results (extract mode).")]
    pub dest: Option<PathBuf>,
    #[clap(short = 'o', long = "output", default_value = "totebag.zip", value_name = "OUTPUT", required = false, help = "Output file (archive mode).")]
    pub output: Option<PathBuf>,
    #[clap(long = "to-archive-name-dir", help = "extract files to DEST/ARCHIVE_NAME directory (extract mode).", default_value_t = false)]
    pub to_archive_name_dir: bool,
    #[clap(short = 'n', long = "no-recursive", help = "No recursive directory (archive mode).", default_value_t = false)]
    pub no_recursive: bool,
    #[clap(short = 'v', long = "verbose", help = "Display verbose output.", default_value_t = false)]
    pub verbose: bool,
    #[clap(long, help = "Overwrite existing files.")]
    pub overwrite: bool,
    #[clap(value_name = "ARGUMENTS", help = "List of files or directories to be processed.")]
    pub args: Vec<PathBuf>,
}

impl CliOpts {
    pub fn run_mode(&mut self) -> Result<RunMode> {
        if self.args.len() == 0 {
            return Err(ToatError::NoArgumentsGiven)
        }
        if self.mode == RunMode::Auto {
            if is_all_args_archives(&self.args) {
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

fn is_all_args_archives(args: &[PathBuf]) -> bool {
    args.iter().all(|arg| {
        let name = arg.to_str().unwrap().to_lowercase();
        name.ends_with(".zip") || name.ends_with(".tar") || name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".rar")
    })
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub enum RunMode {
    Auto,
    Archive,
    Extract,
    List,
}

#[derive(Debug)]
pub enum ToatError {
    NoArgumentsGiven,
    FileNotFound(PathBuf),
    FileExists(PathBuf),
    IOError(std::io::Error),
    ArchiverError(String),
    UnsupportedFormat(String),
    UnknownError(String),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
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
        assert_eq!(cli3.run_mode().unwrap(), RunMode::Extract);

        let mut cli4 = CliOpts::parse_from(&["totebag_test", "src.zip", "LICENSE.tar", "README.tar.bz2", "hoge.rar", "--mode", "list"]);
        let r4 = cli3.run_mode();
        assert_eq!(cli4.run_mode().unwrap(), RunMode::List);
    }

    #[test]
    fn test_is_all_args_archives() {
        assert!(is_all_args_archives(&[PathBuf::from("test.zip"), PathBuf::from("test.tar"), PathBuf::from("test.tar.gz"), PathBuf::from("test.tgz"), PathBuf::from("test.tar.bz2"), PathBuf::from("test.tbz2"), PathBuf::from("test.rar")]));
    }
}
