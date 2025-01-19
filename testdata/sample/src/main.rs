use clap::Parser;
use cli::*;
use cli::{ToatError, RunMode};
use archiver::{archiver_info, ArchiverOpts};
use extractor::{create_extract_opts, extractor_info};

mod cli;
mod format;
mod archiver;
mod extractor;
mod verboser;

fn perform(mut opts: CliOpts) -> Result<()> {
    match opts.run_mode() {
        Ok(RunMode::Archive) => {
            return perform_archive(opts)
        }
        Ok(RunMode::Extract) => {
            return perform_extract(opts)
        }
        Ok(RunMode::List) => {
            return perform_list(opts)
        }
        Ok(RunMode::Auto) => {
            return Err(ToatError::UnknownError("cannot distinguish archiving and extracting".to_string()))
        }
        Err(e) => {
            return Err(e);
        }
    };
}

fn perform_extract(opts: CliOpts) -> Result<()> {
    let args = opts.args.clone();
    let extract_opts = create_extract_opts(opts);
    for arg in args.iter() {
        let extractor = extractor::create_extractor(arg).unwrap();
        let target = arg.to_path_buf();
        extract_opts.v.verbose(extractor_info(&extractor, &target, &extract_opts));
        extractor.perform(target, &extract_opts)?;
    };
    Ok(())
}

fn perform_list(opts: CliOpts) -> Result<()> {
    let args = opts.args.clone();
    for arg in args.iter() {
        if !arg.exists() {
            return Err(ToatError::FileNotFound(arg.to_path_buf()))
        }
        let extractor = extractor::create_extractor(arg).unwrap();
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
    let archiver = archiver::create_archiver(opts.output.clone().unwrap()).unwrap();
    let inout = ArchiverOpts::new(&opts);
    inout.v.verbose(archiver_info(&archiver, &inout));
    archiver.perform(inout)
}

fn main() -> Result<()> {
    match perform(CliOpts::parse()) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            Err(e)
        }
    
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use cli::RunMode;
    use super::*;

    #[test]
    fn test_run() {
        let opts = CliOpts::parse_from(&["totebag_test", "-o", "test.zip", "src", "LICENSE", "README.md", "Cargo.toml"]);
        assert_eq!(opts.mode, RunMode::Auto);
        assert_eq!(opts.output, Some(PathBuf::from("test.zip")));
        assert_eq!(opts.args.len(), 4);
        assert_eq!(opts.args, vec![PathBuf::from("src"), PathBuf::from("LICENSE"), PathBuf::from("README.md"), PathBuf::from("Cargo.toml")]);
    }
}