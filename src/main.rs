use clap::Parser;
use cli::*;
use cli::{ToatError, RunMode};
use archiver::InOut;

mod cli;
mod archiver;

fn perform(mut opts: CliOpts) -> Result<()> {
    match opts.run_mode() {
        Ok(RunMode::Archive) => {
            return perform_archive(opts)
        }
        Ok(RunMode::Extract) => {
            println!("Extracting...");
            // archiver::extract(&opts);
        }
        Ok(RunMode::Auto) => {
            return Err(ToatError::UnknownError("cannot distinguish archiving and extracting".to_string()))
        }
        Err(e) => {
            return Err(e);
        }
    };
    Ok(())
}

fn perform_archive(opts: CliOpts) -> Result<()> {
    let archiver = archiver::create_archiver(opts.output.clone().unwrap()).unwrap();
    let output = opts.output.unwrap();
    let args = opts.args; // Clone the opts.args vector
    let inout = InOut::new(output, args, opts.overwrite, !opts.no_recursive);
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