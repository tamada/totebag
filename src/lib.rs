pub mod archiver;
pub mod extractor;
pub mod format;

use clap::ValueEnum;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, ToteError>;

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy, Hash, Eq)]
pub enum IgnoreType {
    Default,
    Hidden,
    GitIgnore,
    GitGlobal,
    GitExclude,
    Ignore,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub enum RunMode {
    Auto,
    Archive,
    Extract,
    List,
}

#[derive(Debug)]
pub enum ToteError {
    Archiver(String),
    Array(Vec<ToteError>),
    DestIsDir(PathBuf),
    DirExists(PathBuf),
    Fatal(Box<dyn std::error::Error>),
    FileNotFound(PathBuf),
    FileExists(PathBuf),
    IO(std::io::Error),
    NoArgumentsGiven,
    Unknown(String),
    UnknownFormat(String),
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::archiver::{Archiver, ArchiverOpts};
    use crate::extractor::{Extractor, ExtractorOpts};
    use crate::format::Format;
    use crate::Result;

    fn archive_file(dest: PathBuf, sources: Vec<PathBuf>) -> Result<()> {
        let opts = ArchiverOpts::new(None, true, true, vec![]);
        let archiver = Archiver::new(dest, sources, &opts);
        archiver.perform()
    }

    fn archive_and_extract(f: Format, dest: PathBuf, sources: Vec<PathBuf>) {
        let r = archive_file(dest.clone(), sources.clone());
        assert!(r.is_ok());
        let opts = ExtractorOpts::new_with_opts(dest, Some(PathBuf::from("results")), false, true);
        let extractor = Extractor::new(&opts);
        assert_eq!(f, opts.format());
        let r = extractor.list();
        assert!(r.is_ok());
        let list = r.unwrap();
        assert!(list.contains(&"testdata/sample/Cargo.toml".to_string()));
        assert!(list.contains(&"testdata/sample/LICENSE".to_string()));
        assert!(list.contains(&"testdata/sample/README.md".to_string()));
        assert!(list.contains(&"testdata/sample/build.rs".to_string()));
    }

    #[test]
    fn test_archive_and_extract() {
        let sources = vec!["testdata/sample"]
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>();
        archive_and_extract(
            Format::Zip,
            PathBuf::from("results/union_test.zip"),
            sources.clone(),
        );
        archive_and_extract(
            Format::Cab,
            PathBuf::from("results/union_test.cab"),
            sources.clone(),
        );
        archive_and_extract(
            Format::SevenZ,
            PathBuf::from("results/union_test.7z"),
            sources.clone(),
        );
        archive_and_extract(
            Format::Tar,
            PathBuf::from("results/union_test.tar"),
            sources.clone(),
        );
        archive_and_extract(
            Format::TarGz,
            PathBuf::from("results/union_test.tar.gz"),
            sources.clone(),
        );
        archive_and_extract(
            Format::TarBz2,
            PathBuf::from("results/union_test.tar.bz2"),
            sources.clone(),
        );
        archive_and_extract(
            Format::TarXz,
            PathBuf::from("results/union_test.tar.xz"),
            sources.clone(),
        );
        archive_and_extract(
            Format::TarZstd,
            PathBuf::from("results/union_test.tar.zst"),
            sources.clone(),
        );
    }
}
