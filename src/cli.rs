use std::path::PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[clap(
    version, author, about,
    arg_required_else_help = true,
)]
pub struct CliOpts {
    #[clap(short = 'm', long = "mode", default_value_t = RunMode::Auto, value_name = "MODE", required = false, ignore_case = true, value_enum, help = "Mode of operation.")]
    pub mode: RunMode,
    #[clap(short = 'd', long = "dest", default_value = ".", value_name = "DEST", required = false, help = "Destination of the extraction results.")]
    pub dest: Option<PathBuf>,
    #[clap(short = 'o', long = "output", default_value = "totebag.zip", value_name = "OUTPUT", required = false, help = "Output file for the archive.")]
    pub output: Option<PathBuf>,
    #[clap(short = 'n', long = "no-recursive", help = "No recursive mode.", default_value_t = false)]
    pub no_recursive: bool,
    #[clap(short = 'v', long = "verbose", help = "Display verbose output.", default_value_t = false)]
    pub verbose: bool,
    #[clap(value_name = "ARGUMENTS", help = "List of files or directories to be processed.")]
    pub args: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum RunMode {
    Auto,
    Archive,
    Extract,
}