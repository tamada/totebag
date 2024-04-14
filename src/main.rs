use clap::Parser;
use cli::CliOpts;

mod cli;

fn main() {
    let _opts = CliOpts::parse();
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