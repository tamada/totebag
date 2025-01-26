use clap::Parser;
use std::path::PathBuf;

use cli::RunMode;
use totebag::archiver::Archiver;
use totebag::extractor::Extractor;
use totebag::{Result, ToteError};

mod cli;
mod list;

fn update_loglevel(opts: &cli::CliOpts) {
    use env_logger;
    match opts.level {
        cli::LogLevel::Error => std::env::set_var("RUST_LOG", "error"),
        cli::LogLevel::Warn => std::env::set_var("RUST_LOG", "warn"),
        cli::LogLevel::Info => std::env::set_var("RUST_LOG", "info"),
        cli::LogLevel::Debug => std::env::set_var("RUST_LOG", "debug"),
    }
    env_logger::init();
}

fn perform(mut opts: cli::CliOpts) -> Result<()> {
    update_loglevel(&opts);
    if cfg!(debug_assertions) {
        #[cfg(debug_assertions)]
        if opts.generate_completion {
            return gencomp::generate(PathBuf::from("target/completions"));
        }
    }
    match opts.run_mode() {
        Ok(RunMode::Archive) => perform_archive(opts),
        Ok(RunMode::Extract) => perform_extract_or_list(opts, perform_extract_each),
        Ok(RunMode::List) => perform_extract_or_list(opts, perform_list_each),
        Ok(RunMode::Auto) => Err(ToteError::Unknown(
            "cannot distinguish archiving and extracting".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn perform_extract_or_list<F>(opts: cli::CliOpts, f: F) -> Result<()>
where
    F: Fn(&cli::CliOpts, PathBuf) -> Result<()>,
{
    let args = opts
        .args
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();
    log::info!("args: {:?}", args);
    let mut errs = vec![];
    for arg in args {
        match f(&opts, arg) {
            Err(e) => errs.push(e),
            Ok(_) => {}
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(ToteError::Array(errs))
    }
}

fn perform_extract_each(opts: &cli::CliOpts, archive_file: PathBuf) -> Result<()> {
    let extractor = Extractor::builder()
        .archive_file(archive_file)
        .destination(opts.output.clone().unwrap_or_else(|| PathBuf::from(".")))
        .use_archive_name_dir(opts.extractors.to_archive_name_dir)
        .overwrite(opts.overwrite)
        .build();
    log::info!("{}", extractor.info());
    extractor.perform()
}

fn perform_list_each(opts: &cli::CliOpts, archive_file: PathBuf) -> Result<()> {
    let extractor = Extractor::builder()
        .archive_file(archive_file)
        .destination(opts.output.clone().unwrap_or_else(|| PathBuf::from(".")))
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

fn perform_archive(cliopts: cli::CliOpts) -> Result<()> {
    let archiver = Archiver::builder()
        .archive_file(cliopts.output.unwrap())
        .targets(
            cliopts
                .args
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<PathBuf>>(),
        )
        .rebase_dir(cliopts.archivers.base_dir)
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

fn print_error(e: &ToteError) {
    match e {
        ToteError::Archiver(s) => println!("Archive error: {}", s),
        ToteError::Array(errs) => {
            for err in errs.into_iter() {
                print_error(err);
            }
        }
        ToteError::DestIsDir(p) => println!("{}: destination is a directory", p.to_str().unwrap()),
        ToteError::DirExists(p) => println!("{}: directory already exists", p.to_str().unwrap()),
        ToteError::Fatal(e) => println!("Error: {}", e),
        ToteError::FileNotFound(p) => println!("{}: file not found", p.to_str().unwrap()),
        ToteError::FileExists(p) => println!("{}: file already exists", p.to_str().unwrap()),
        ToteError::IO(e) => println!("IO error: {}", e),
        ToteError::NoArgumentsGiven => println!("No arguments given. Use --help for usage."),
        ToteError::Unknown(s) => println!("Unknown error: {}", s),
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
        assert_eq!(opts.mode, RunMode::Auto);
        assert_eq!(opts.output, Some(PathBuf::from("test.zip")));
        assert_eq!(opts.args.len(), 4);
        assert_eq!(opts.args, vec!["src", "LICENSE", "README.md", "Cargo.toml"]);
    }
}
