use std::path::PathBuf;
use clap::Parser;

use totebag::{Result, RunMode, ToteError};
use totebag::archiver::{Archiver, ArchiverOpts};
use totebag::extractor::{Extractor, ExtractorOpts};

mod cli;

fn perform(mut opts: cli::CliOpts) -> Result<()> {
    match opts.run_mode() {
        Ok(RunMode::Archive) => return perform_archive(opts),
        Ok(RunMode::Extract) => return perform_extract(opts),
        Ok(RunMode::List) => return perform_list(opts),
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

fn perform_extract(opts: cli::CliOpts) -> Result<()> {
    let args = opts.args.iter()
        .map(PathBuf::from).collect::<Vec<PathBuf>>();
    println!("args: {:?}", args);
    let extractor_opts = ExtractorOpts::new_with_opts(opts.output, opts.extractors.to_archive_name_dir, opts.overwrite);
    let mut errs = vec![];
    for arg in args.iter() {
        let extractor = match Extractor::new(arg.clone(), &extractor_opts) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        log::info!("{}", extractor.info());
        if let Err(e) = match extractor.can_extract() {
            Ok(_) => 
                extractor.perform(),
            Err(e) => Err(e),
        } {
            errs.push(e);
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(ToteError::Array(errs))
    }
}

fn perform_list(opts: cli::CliOpts) -> Result<()> {
    let args = opts.args.clone();
    let extractor_opts = ExtractorOpts::new_with_opts(opts.output, opts.extractors.to_archive_name_dir, opts.overwrite);
    let mut errs = vec![];
    for arg in args.iter() {
        let path = PathBuf::from(arg);
        if !path.exists() {
            return Err(ToteError::FileNotFound(path));
        }
        let extractor = match Extractor::new(path, &extractor_opts) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        if args.len() > 1 {
            println!("========== {:?} ========== \n", arg);
        }
        match extractor.list() {
            Ok(files) => {
                for file in files {
                    println!("{}", file);
                }
            }
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(ToteError::Array(errs))
    }
}

fn perform_archive(opts: cli::CliOpts) -> Result<()> {
    let inout = ArchiverOpts::new(Some(opts.archivers.base_dir), opts.overwrite, !opts.archivers.no_recursive, opts.archivers.ignores);
    match Archiver::create(opts.args, opts.output, inout) {
        Ok(archiver) => {
            log::info!("{}", archiver.info());
            archiver.perform()
        }
        Err(e) => Err(e),
    }
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
        ToteError::FileExists(p) => 
            println!("{}: file already exists", p.to_str().unwrap()),
        ToteError::IO(e) => println!("IO error: {}", e),
        ToteError::NoArgumentsGiven => 
            println!("No arguments given. Use --help for usage."),
        ToteError::Unknown(s) => println!("Unknown error: {}", s),
        ToteError::UnknownFormat(f) => println!("{}: unknown format", f),
        ToteError::UnsupportedFormat(f) => println!("{}: unsupported format", f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use totebag::RunMode;
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
        assert_eq!(
            opts.args,
            vec![
                "src",
                "LICENSE",
                "README.md",
                "Cargo.toml"
            ]
        );
    }
}
