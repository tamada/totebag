use archiver::{archiver_info, ArchiverOpts};
use clap::Parser;
use cli::*;
use cli::{RunMode, ToteError};
use extractor::{extractor_info, ExtractorOpts};
use format::is_all_args_archives;

mod archiver;
mod cli;
mod extractor;
mod format;
mod verboser;

impl CliOpts {
    pub fn run_mode(&mut self) -> Result<RunMode> {
        if self.args.len() == 0 {
            return Err(ToteError::NoArgumentsGiven)
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

fn perform(mut opts: CliOpts) -> Result<()> {
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

fn perform_extract(opts: CliOpts) -> Result<()> {
    let args = opts.args.clone();
    let extract_opts = ExtractorOpts::new(&opts);
    for arg in args.iter() {
        let extractor = extractor::create_extractor(arg).unwrap();
        let target = arg.to_path_buf();
        extract_opts
            .v
            .verbose(extractor_info(&extractor, &target, &extract_opts));
        extractor.perform(target, &extract_opts)?;
    }
    Ok(())
}

fn perform_list(opts: CliOpts) -> Result<()> {
    let args = opts.args.clone();
    for arg in args.iter() {
        if !arg.exists() {
            return Err(ToteError::FileNotFound(arg.to_path_buf()));
        }
        let extractor = extractor::create_extractor(&arg).unwrap();
        if args.len() > 1 {
            println!("========== {:?} ========== \n", arg);
        }
        let files = extractor.list_archives(arg.to_path_buf()).unwrap();
        for file in files.iter() {
            println!("{}", file);
        }
    }
    Ok(())
}

fn perform_archive(opts: CliOpts) -> Result<()> {
    let inout = ArchiverOpts::new(&opts);
    match archiver::create_archiver(&inout.dest) {
        Ok(archiver) => {
            inout.v.verbose(archiver_info(&archiver, &inout));
            archiver.perform(&inout)
        }
        Err(e) => Err(e),
    }
}

fn main() -> Result<()> {
    match perform(CliOpts::parse()) {
        Ok(_) => Ok(()),
        Err(e) => {
            match e {
                ToteError::NoArgumentsGiven => {
                    println!("No arguments given. Use --help for usage.")
                }
                ToteError::FileNotFound(p) => println!("{}: file not found", p.to_str().unwrap()),
                ToteError::FileExists(p) => {
                    println!("{}: file already exists", p.to_str().unwrap())
                }
                ToteError::IO(e) => println!("IO error: {}", e),
                ToteError::Archiver(s) => println!("Archive error: {}", s),
                ToteError::UnknownFormat(f) => println!("{}: unknown format", f),
                ToteError::UnsupportedFormat(f) => println!("{}: unsupported format", f),
                ToteError::Fatal(e) => println!("Error: {}", e),
                ToteError::Unknown(s) => println!("Unknown error: {}", s),
            }
            std::process::exit(1);
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
        let opts = CliOpts::parse_from(&[
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
                PathBuf::from("src"),
                PathBuf::from("LICENSE"),
                PathBuf::from("README.md"),
                PathBuf::from("Cargo.toml")
            ]
        );
    }
}
