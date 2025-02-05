use clap::Parser;
use std::path::PathBuf;

use cli::{LogLevel, RunMode};
use totebag::archiver::{ArchiveEntries, Archiver};
use totebag::extractor::Extractor;
use totebag::format::Manager as FormatManager;
use totebag::{Result, ToteError};

mod cli;
mod list;

fn update_loglevel(level: LogLevel) {
    match level {
        cli::LogLevel::Error => std::env::set_var("RUST_LOG", "error"),
        cli::LogLevel::Warn => std::env::set_var("RUST_LOG", "warn"),
        cli::LogLevel::Info => std::env::set_var("RUST_LOG", "info"),
        cli::LogLevel::Debug => std::env::set_var("RUST_LOG", "debug"),
        cli::LogLevel::Trace => std::env::set_var("RUST_LOG", "trace"),
    }
    env_logger::init();
    log::info!("set log level to {:?}", level);
}

fn perform(mut opts: cli::CliOpts) -> Result<()> {
    update_loglevel(opts.loglevel);
    if cfg!(debug_assertions) {
        #[cfg(debug_assertions)]
        if opts.generate_completion {
            return gencomp::generate(PathBuf::from("target/completions"));
        }
    }
    let manager = FormatManager::default();
    opts.finalize(&manager)?;
    match opts.run_mode() {
        RunMode::Archive => match perform_archive(opts, manager) {
            Ok(result) => {
                print_archive_result(result);
                Ok(())
            }
            Err(e) => Err(e),
        },
        RunMode::Extract => perform_extract_or_list(opts, manager, perform_extract_each),
        RunMode::List => perform_extract_or_list(opts, manager, perform_list_each),
        RunMode::Auto => Err(ToteError::Warn(
            "cannot distinguish archiving and extracting".to_string(),
        )),
    }
}

fn perform_extract_or_list<F>(opts: cli::CliOpts, m: FormatManager, f: F) -> Result<()>
where
    F: Fn(&cli::CliOpts, FormatManager, PathBuf) -> Result<()>,
{
    let args = opts
        .args()
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();
    log::info!("args: {:?}", args);
    let mut errs = vec![];
    for arg in args {
        if let Err(e) = f(&opts, m.clone(), arg) {
            errs.push(e);
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(ToteError::Array(errs))
    }
}

fn perform_extract_each(
    opts: &cli::CliOpts,
    fm: FormatManager,
    archive_file: PathBuf,
) -> Result<()> {
    let extractor = Extractor::builder()
        .archive_file(archive_file)
        .manager(fm)
        .destination(opts.extractor_output())
        .use_archive_name_dir(opts.extractors.to_archive_name_dir)
        .overwrite(opts.overwrite)
        .build();
    log::info!("{}", extractor.info());
    extractor.perform()
}

fn perform_list_each(opts: &cli::CliOpts, fm: FormatManager, archive_file: PathBuf) -> Result<()> {
    let extractor = Extractor::builder()
        .archive_file(archive_file)
        .manager(fm)
        .destination(opts.extractor_output())
        .use_archive_name_dir(opts.extractors.to_archive_name_dir)
        .overwrite(opts.overwrite)
        .build();
    log::info!("{}", extractor.info());
    match extractor.list() {
        Ok(files) => {
            for file in files {
                if opts.listers.long {
                    list::print_long_format(file)
                } else {
                    println!("{}", file.name);
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn perform_archive(cliopts: cli::CliOpts, fm: FormatManager) -> Result<ArchiveEntries> {
    if cliopts.output.is_none() {
        return Err(ToteError::Archiver(
            "output file is not specified".to_string(),
        ));
    }
    let archiver = Archiver::builder()
        .archive_file(cliopts.archiver_output())
        .manager(fm.clone())
        .targets(
            cliopts
                .args()
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<PathBuf>>(),
        )
        .rebase_dir(cliopts.archivers.base_dir)
        .level(cliopts.archivers.level)
        .overwrite(cliopts.overwrite)
        .no_recursive(cliopts.archivers.no_recursive)
        .ignore_types(cliopts.archivers.ignores)
        .build();
    log::info!("{}", archiver.info());
    archiver.perform()
}

fn main() -> Result<()> {
    if let Err(e) = perform(cli::CliOpts::parse()) {
        print_error(&e);
        std::process::exit(1);
    }
    Ok(())
}

fn print_archive_result(result: ArchiveEntries) {
    if log::log_enabled!(log::Level::Info) {
        list::print_archive_result(result);
    }
}

fn print_error(e: &ToteError) {
    match e {
        ToteError::Archiver(s) => println!("Archive error: {}", s),
        ToteError::Array(errs) => {
            for err in errs.iter() {
                print_error(err);
            }
        }
        ToteError::DestIsDir(p) => println!("{}: destination is a directory", p.to_str().unwrap()),
        ToteError::DirExists(p) => println!("{}: directory already exists", p.to_str().unwrap()),
        ToteError::Extractor(s) => println!("Extractor error: {}", s),
        ToteError::Fatal(e) => println!("Error: {}", e),
        ToteError::FileNotFound(p) => println!("{}: file not found", p.to_str().unwrap()),
        ToteError::FileExists(p) => println!("{}: file already exists", p.to_str().unwrap()),
        ToteError::IO(e) => println!("IO error: {}", e),
        ToteError::NoArgumentsGiven => println!("No arguments given. Use --help for usage."),
        ToteError::Warn(s) => println!("Unknown error: {}", s),
        ToteError::UnknownFormat(f) => println!("{}: unknown format", f),
        ToteError::UnsupportedFormat(f) => println!("{}: unsupported format", f),
    }
}

#[cfg(debug_assertions)]
mod gencomp {
    use crate::cli::CliOpts;
    use totebag::{Result, ToteError};

    use clap::{Command, CommandFactory};
    use clap_complete::Shell;
    use std::path::PathBuf;

    fn generate_impl(app: &mut Command, shell: Shell, dest: PathBuf) -> Result<()> {
        log::info!("generate completion for {:?} to {:?}", shell, dest);
        if let Err(e) = std::fs::create_dir_all(dest.parent().unwrap()) {
            return Err(ToteError::IO(e));
        }
        match std::fs::File::create(dest) {
            Err(e) => Err(ToteError::IO(e)),
            Ok(mut out) => {
                clap_complete::generate(shell, app, "totebag", &mut out);
                Ok(())
            }
        }
    }

    pub fn generate(outdir: PathBuf) -> Result<()> {
        let shells = vec![
            (Shell::Bash, "bash/totebag"),
            (Shell::Fish, "fish/totebag"),
            (Shell::Zsh, "zsh/_totebag"),
            (Shell::Elvish, "elvish/totebag"),
            (Shell::PowerShell, "powershell/totebag"),
        ];
        let mut app = CliOpts::command();
        app.set_bin_name("totebag");
        let mut errs = vec![];
        for (shell, file) in shells {
            if let Err(e) = generate_impl(&mut app, shell, outdir.join(file)) {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(ToteError::Array(errs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::RunMode;
    use std::path::PathBuf;

    #[test]
    fn test_run() {
        let opts = cli::CliOpts::parse_from(&[
            "totebag_test",
            "-o",
            "test.zip",
            "src",
            "LICENSE",
            "README.md",
            "Cargo.toml",
        ]);
        let args = opts.args();
        assert_eq!(opts.mode, RunMode::Auto);
        assert_eq!(opts.output, Some(PathBuf::from("test.zip")));
        assert_eq!(args.len(), 4);
        assert_eq!(args, vec!["src", "LICENSE", "README.md", "Cargo.toml"]);
    }

    #[test]
    fn test_update_loglevel_error() {
        update_loglevel(LogLevel::Error);
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "error");
    }
}
