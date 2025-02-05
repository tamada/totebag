use std::path::PathBuf;

use totebag::archiver::{ArchiveEntries, Archiver};
use totebag::extractor::Extractor;
use totebag::format::Format;
use totebag::Result;

fn archive_file(dest: PathBuf, sources: Vec<PathBuf>) -> Result<ArchiveEntries> {
    let archiver = Archiver::builder()
        .archive_file(dest)
        .targets(sources)
        .overwrite(true)
        .build();
    archiver.perform()
}

fn archive_and_extract(f: Format, archive_file_name: PathBuf, sources: Vec<PathBuf>) {
    let r = archive_file(archive_file_name.clone(), sources);
    assert!(r.is_ok());
    let e = Extractor::builder()
        .archive_file(archive_file_name.clone())
        .destination(PathBuf::from("results"))
        .build();
    match e.format() {
        Some(format) => assert_eq!(&f, format),
        None => panic!("unexpected error: {:?}", archive_file_name),
    }
    let r = e.list();
    assert!(r.is_ok());
    let list = r
        .unwrap()
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<String>>();
    println!("{:?}", list);
    assert!(list.contains(&"testdata/sample/Cargo.toml".to_string()));
    assert!(list.contains(&"testdata/sample/LICENSE".to_string()));
    assert!(list.contains(&"testdata/sample/README.md".to_string()));
    assert!(list.contains(&"testdata/sample/build.rs".to_string()));
}

fn gen_sources() -> Vec<PathBuf> {
    vec!["testdata/sample"]
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>()
}

#[test]
fn test_archive_and_extract_zip() {
    archive_and_extract(
        Format::new("Zip", vec![".zip", ".jar", ".war", ".ear"]),
        PathBuf::from("results/union_test.zip"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_cab() {
    archive_and_extract(
        Format::new("Cab", vec![".cab"]),
        PathBuf::from("results/union_test.cab"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_sevenz() {
    archive_and_extract(
        Format::new("SevenZ", vec![".7z"]),
        PathBuf::from("results/union_test.7z"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_tar() {
    archive_and_extract(
        Format::new("Tar", vec![".tar"]),
        PathBuf::from("results/union_test.tar"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_targz() {
    archive_and_extract(
        Format::new("TarGz", vec![".tar.gz", ".tgz"]),
        PathBuf::from("results/union_test.tar.gz"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_tarbz2() {
    archive_and_extract(
        Format::new("TarBz2", vec![".tar.bz2", ".tbz2"]),
        PathBuf::from("results/union_test.tar.bz2"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_tarxz() {
    archive_and_extract(
        Format::new("TarXz", vec![".tar.xz", ".txz"]),
        PathBuf::from("results/union_test.tar.xz"),
        gen_sources(),
    );
}
#[test]
fn test_archive_and_extract_tarzstd() {
    archive_and_extract(
        Format::new("TarZstd", vec![".tar.zst", ".tzst", ".tar.zstd", ".tzstd"]),
        PathBuf::from("results/union_test.tar.zst"),
        gen_sources(),
    );
}
