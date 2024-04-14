use std::path::PathBuf;
use clap::{Parser, ValueEnum};

mod archiver;

#[derive(Parser)]
#[clap(
    version, author, about,
    arg_required_else_help = true,
)]
struct CliOpts {
    #[clap(short = 'm', long = "mode", default_value_t = RunMode::Auto, value_name = "MODE", required = false, ignore_case = true, value_enum, help = "Mode of operation.")]
    mode: RunMode,
    #[clap(short = 'd', long = "dest", default_value = ".", value_name = "DEST", required = false, help = "Destination of the extraction results.")]
    dest: Option<PathBuf>,
    #[clap(short = 'o', long = "output", default_value = "totebag.zip", value_name = "OUTPUT", required = false, help = "Output file for the archive.")]
    output: Option<PathBuf>,
    #[clap(short = 'n', long = "no-recursive", help = "No recursive mode.", default_value_t = false)]
    no_recursive: bool,
    #[clap(short = 'v', long = "verbose", help = "Display verbose output.", default_value_t = false)]
    verbose: bool,
    #[clap(value_name = "ARGUMENTS", help = "List of files or directories to be processed.")]
    args: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum RunMode {
    Extract,
    Archive,
    Auto,
}


fn main() {
    let _opts: CliOpts = CliOpts::parse();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_args_parse() {
        let args = CliOpts::try_parse_from(&[
            "totebag", "-o", "output.zip", "src", "LICENSE", "README.md", "Cargo.toml",
        ]);
        assert!(args.is_ok());
        assert_eq!(args.unwrap().args.len(), 4)
    }
}

