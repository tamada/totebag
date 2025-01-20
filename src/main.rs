use clap::Parser;
use std::path::PathBuf;

use totebag::archiver::{Archiver, ArchiverOpts};
use totebag::extractor::{Extractor, ExtractorOpts};
use totebag::{Result, RunMode, ToteError};

mod cli;

fn perform(mut opts: cli::CliOpts) -> Result<()> {
    match opts.run_mode() {
        Ok(RunMode::Archive) => return perform_archive(opts),
        Ok(RunMode::Extract) => return perform_extract_or_list(opts, perform_extract_each),
        Ok(RunMode::List) => return perform_extract_or_list(opts, perform_list_each),
        Ok(RunMode::Auto) => {
            return Err(ToteError::Unknown(
                "cannot distinguish archiving and extracting".to_string(),
            ))
        }
        Err(e) => {
            return Err(e);
        }
    };
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

fn perform_extract(opts: cli::CliOpts) -> Result<()> {
    let args = opts
        .args
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();
    log::info!("args: {:?}", args);
    let mut errs = vec![];
    for arg in args {
        match perform_extract_each(&opts, arg) {
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

fn perform_extract_each(opts: &cli::CliOpts, arg: PathBuf) -> Result<()> {
    let extractor_opts = ExtractorOpts::new_with_opts(
        arg,
        opts.output.clone(),
        opts.extractors.to_archive_name_dir,
        opts.overwrite,
    );
    let extractor = Extractor::new(&extractor_opts);
    log::info!("{}", extractor.info());
    match extractor_opts.can_extract() {
        Ok(_) => extractor.perform(),
        Err(e) => Err(e),
    }
}

fn perform_list_each(opts: &cli::CliOpts, arg: PathBuf) -> Result<()> {
    let extractor_opts = ExtractorOpts::new_with_opts(
        arg,
        opts.output.clone(),
        opts.extractors.to_archive_name_dir,
        opts.overwrite,
    );
    let extractor = Extractor::new(&extractor_opts);
    log::info!("{}", extractor.info());
    match extractor.list() {
        Ok(files) => {
            for file in files {
                println!("{}", file);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn perform_archive(cliopts: cli::CliOpts) -> Result<()> {
    let opts = ArchiverOpts::new(
        Some(cliopts.archivers.base_dir),
        cliopts.overwrite,
        !cliopts.archivers.no_recursive,
        cliopts.archivers.ignores,
    );
    let archiver = Archiver::create(cliopts.args, cliopts.output, &opts);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use totebag::RunMode;

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
