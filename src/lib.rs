pub mod archiver;
pub mod extractor;
pub mod format;

use clap::ValueEnum;
use std::path::PathBuf;

/// Define the result type for the this library.
pub type Result<T> = std::result::Result<T, ToteError>;

/// Define the ignore types for directory traversing.
#[derive(Debug, Clone, ValueEnum, PartialEq, Copy, Hash, Eq)]
pub enum IgnoreType {
    Default,
    Hidden,
    GitIgnore,
    GitGlobal,
    GitExclude,
    Ignore,
}

/// Define the errors for this library.
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
    use crate::extractor::create;
    use crate::format::{find_format, Format};
    use crate::Result;

    fn archive_file(dest: PathBuf, sources: Vec<PathBuf>) -> Result<()> {
        let opts = ArchiverOpts::new(None, true, true, vec![]);
        let archiver = Archiver::new(dest, sources, &opts);
        archiver.perform()
    }

    fn archive_and_extract(f: Format, archive_file_name: PathBuf, sources: Vec<PathBuf>) {
        let r = archive_file(archive_file_name.clone(), sources.clone());
        assert!(r.is_ok());
        let e = create(archive_file_name.clone()).unwrap();
        match find_format(&archive_file_name) {
            Ok(format) => assert_eq!(f, format),
            Err(e) => panic!("unexpected error: {:?}", e),
        }
        let r = e.list();
        assert!(r.is_ok());
        let list = r
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect::<Vec<String>>();
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
