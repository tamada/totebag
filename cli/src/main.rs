use clap::Parser;
use std::path::PathBuf;

use cli::LogLevel;
use totebag::archiver::ArchiveEntries;
use totebag::{Result, ToteError};

use crate::cli::Mode;

mod cli;
mod list;

fn update_loglevel(level: LogLevel) {
    unsafe {
        match level {
            cli::LogLevel::Error => std::env::set_var("RUST_LOG", "error"),
            cli::LogLevel::Warn => std::env::set_var("RUST_LOG", "warn"),
            cli::LogLevel::Info => std::env::set_var("RUST_LOG", "info"),
            cli::LogLevel::Debug => std::env::set_var("RUST_LOG", "debug"),
            cli::LogLevel::Trace => std::env::set_var("RUST_LOG", "trace"),
        }
    }
    env_logger::try_init().unwrap_or_else(|_| {
        eprintln!("failed to initialize logger. set RUST_LOG to see logs.");
    });
    log::info!("set log level to {level:?}");
}

fn perform(opts: cli::CliOpts) -> Result<()> {
    update_loglevel(opts.loglevel);
    if cfg!(debug_assertions) {
        #[cfg(debug_assertions)]
        if opts.generate_completion {
            return gencomp::generate(PathBuf::from("target/completions"));
        }
    }
    let (mode, args) = opts.find_mode()?;
    match mode {
        Mode::Archive(config) => match perform_archive(config, args) {
            Ok(entries) => print_archive_result(entries),
            Err(e) => Err(e),
        },
        Mode::List(config) => match perform_list(config, args) {
            Ok(results) => print_list_result(results),
            Err(e) => Err(e),
        },
        Mode::Extract(config) => perform_extract(config, args),
    }
}

fn perform_extract(config: totebag::ExtractConfig, args: Vec<String>) -> Result<()> {
    let mut errs = vec![];
    for item in args {
        let path = PathBuf::from(item);
        if let Err(e) = totebag::extract(path, &config) {
            errs.push(e);
        }
    }
    ToteError::error_or((), errs)
}

fn perform_list(config: totebag::ListConfig, args: Vec<String>) -> Result<Vec<String>> {
    let mut errs = vec![];
    let mut results = vec![];
    for item in args {
        let path = PathBuf::from(item);
        match totebag::list(path, &config) {
            Ok(r) => results.push(r),
            Err(e) => errs.push(e),
        }
    }
    ToteError::error_or(results, errs)
}

fn perform_archive(config: totebag::ArchiveConfig, args: Vec<String>) -> Result<ArchiveEntries> {
    let targets = args.into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    totebag::archive(&targets, &config)
}

fn main() -> Result<()> {
    if let Err(e) = perform(cli::CliOpts::parse()) {
        print_error(&e);
        std::process::exit(1);
    }
    Ok(())
}

fn print_list_result(results: Vec<String>) -> Result<()> {
    results.iter().for_each(|item| println!("{item}"));
    Ok(())
}

fn print_archive_result(result: ArchiveEntries) -> Result<()> {
    if log::log_enabled!(log::Level::Info) {
        list::print_archive_result(result);
    }
    Ok(())
}

fn print_error(e: &ToteError) {
    match e {
        ToteError::Archiver(s) => println!("Archive error: {s}"),
        ToteError::Array(errs) => {
            for err in errs.iter() {
                print_error(err);
            }
        }
        ToteError::DestIsDir(p) => println!("{}: destination is a directory", p.to_str().unwrap()),
        ToteError::DirExists(p) => println!("{}: directory already exists", p.to_str().unwrap()),
        ToteError::Extractor(s) => println!("Extractor error: {s}"),
        ToteError::Fatal(e) => println!("Error: {e}"),
        ToteError::FileNotFound(p) => println!("{}: file not found", p.to_str().unwrap()),
        ToteError::FileExists(p) => println!("{}: file already exists", p.to_str().unwrap()),
        ToteError::IO(e) => println!("IO error: {e}"),
        ToteError::Json(e) => println!("Json error: {e}"),
        ToteError::NoArgumentsGiven => println!("No arguments given. Use --help for usage."),
        ToteError::Warn(s) => println!("Unknown error: {s}"),
        ToteError::UnknownFormat(f) => println!("{f}: unknown format"),
        ToteError::UnsupportedFormat(f) => println!("{f}: unsupported format"),
        ToteError::Xml(e) => println!("xml error: {e}"),
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
        log::info!("generate completion for {shell:?} to {dest:?}");
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
        assert_eq!(opts.mode, RunMode::Auto);
        assert_eq!(opts.output, Some(PathBuf::from("test.zip")));
        assert_eq!(opts.args.len(), 4);
        assert_eq!(opts.args, vec!["src", "LICENSE", "README.md", "Cargo.toml"]);
    }

    #[test]
    fn test_list() {
        let opts =
            cli::CliOpts::parse_from(&["totebag_test", "--mode", "list", "../testdata/test.zip"]);
        match perform(opts) {
            Ok(_) => (),
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    /// This test sometimes fails because of the timing of the log initialization.
    /// This test wants to run after other tests are run.
    #[test]
    #[ignore]
    fn test_update_loglevel_error() {
        update_loglevel(LogLevel::Error);
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "error");
    }
}
