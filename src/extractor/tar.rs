use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use crate::{Result, ToteError};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extractor::{Entry as ToteEntry, Extractor, ToteExtractor};
use crate::format::Format;

pub(super) struct TarExtractor {
    target: PathBuf,
}

impl TarExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

pub(super) struct TarGzExtractor {
    target: PathBuf,
}

impl TarGzExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

pub(super) struct TarBz2Extractor {
    target: PathBuf,
}

impl TarBz2Extractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

pub(super) struct TarXzExtractor {
    target: PathBuf,
}

impl TarXzExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

pub(super) struct TarZstdExtractor {
    target: PathBuf,
}

impl TarZstdExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

impl ToteExtractor for TarExtractor {
    fn list(&self) -> Result<Vec<ToteEntry>> {
        match open_tar_file(&self.target, |f| f) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, opts: &Extractor) -> Result<()> {
        match open_tar_file(&self.target, |f| f) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, opts),
        }
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}

impl ToteExtractor for TarGzExtractor {
    fn list(&self) -> Result<Vec<ToteEntry>> {
        match open_tar_file(&self.target, |f| flate2::read::GzDecoder::new(f)) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, opts: &Extractor) -> Result<()> {
        match open_tar_file(&self.target, |f| flate2::read::GzDecoder::new(f)) {
            Ok(archive) => extract_tar(archive, opts),
            Err(e) => Err(e),
        }
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}

impl ToteExtractor for TarBz2Extractor {
    fn list(&self) -> Result<Vec<ToteEntry>> {
        match open_tar_file(&self.target, |f| bzip2::read::BzDecoder::new(f)) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, opts: &Extractor) -> Result<()> {
        match open_tar_file(&self.target, |f| bzip2::read::BzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, opts),
        }
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}

impl ToteExtractor for TarXzExtractor {
    fn list(&self) -> Result<Vec<ToteEntry>> {
        match open_tar_file(&self.target, |f| XzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive),
        }
    }
    fn perform(&self, opts: &Extractor) -> Result<()> {
        match open_tar_file(&self.target, |f| XzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, opts),
        }
    }
    fn format(&self) -> Format {
        Format::TarXz
    }
}

impl ToteExtractor for TarZstdExtractor {
    fn list(&self) -> Result<Vec<ToteEntry>> {
        match open_tar_file(&self.target, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive),
        }
    }
    fn perform(&self, opts: &Extractor) -> Result<()> {
        match open_tar_file(&self.target, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, opts),
        }
    }
    fn format(&self) -> Format {
        Format::TarZstd
    }
}

fn open_tar_file<F, R: Read>(file: &PathBuf, opener: F) -> Result<Archive<R>>
where
    F: FnOnce(File) -> R,
{
    let file = match File::open(file) {
        Ok(f) => f,
        Err(e) => return Err(ToteError::IO(e)),
    };
    let writer = opener(file);
    Ok(Archive::new(writer))
}

fn extract_tar<R: Read>(mut archive: tar::Archive<R>, opts: &Extractor) -> Result<()> {
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.header().path().unwrap();
        let p = path.clone().to_path_buf();
        if is_filename_mac_finder_file(p.to_path_buf()) {
            continue;
        }
        let size = entry.header().size().unwrap();
        log::info!("extracting {:?} ({} bytes)", path, size);

        let dest = opts.base_dir().join(path);
        if entry.header().entry_type().is_file() {
            create_dir_all(dest.parent().unwrap()).unwrap();
            entry.unpack(dest).unwrap();
        }
    }
    Ok(())
}

fn is_filename_mac_finder_file(path: PathBuf) -> bool {
    let filename = path.file_name().unwrap().to_str().unwrap();
    filename == ".DS_Store" || filename.starts_with("._")
}

fn list_tar<R: Read>(mut archive: tar::Archive<R>) -> Result<Vec<ToteEntry>> {
    let mut result = vec![];
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        result.push(tar_entry_to_entry(entry));
    }
    Ok(result)
}

fn tar_entry_to_entry<R: Read>(e: tar::Entry<R>) -> ToteEntry {
    let header = e.header();
    let path = header.path().unwrap().to_str().unwrap().to_string();
    let size = header.size();
    let mode = header.mode().unwrap();
    let mtime = header.mtime().unwrap();
    let datetime = chrono::DateTime::from_timestamp_millis(mtime as i64);
    ToteEntry::new(
        path,
        None,
        size.ok(),
        Some(mode),
        datetime.map(|dt| dt.naive_local()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tar_file() {
        let file = PathBuf::from("testdata/test.tar");
        let extractor = TarExtractor::new(file);
        match extractor.list() {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("testdata/test.tar");
        let opts = Extractor::builder()
            .archive_file(archive_file)
            .destination("results/tar")
            .build();
        match opts.perform() {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/tar/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tar")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_list_tarbz2_file() {
        let file = PathBuf::from("testdata/test.tar.bz2");
        let extractor = TarBz2Extractor::new(file);
        match extractor.list() {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_list_targz_file() {
        let file = PathBuf::from("testdata/test.tar.gz");
        let extractor = TarGzExtractor::new(file);
        match extractor.list() {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_list_tarzstd_file() {
        let file = PathBuf::from("testdata/test.tar.zst");
        let extractor = TarZstdExtractor::new(file);
        match extractor.list() {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_format() {
        let e1 = TarExtractor::new(PathBuf::from("testdata/test.tar"));
        assert_eq!(e1.format(), Format::Tar);

        let e2 = TarGzExtractor::new(PathBuf::from("testdata/test.tar.xz"));
        assert_eq!(e2.format(), Format::TarGz);

        let e3 = TarBz2Extractor::new(PathBuf::from("testdata/test.tar.xz"));
        assert_eq!(e3.format(), Format::TarBz2);

        let e4 = TarXzExtractor::new(PathBuf::from("testdata/test.tar.xz"));
        assert_eq!(e4.format(), Format::TarXz);

        let e5 = TarZstdExtractor::new(PathBuf::from("testdata/test.tar.zst"));
        assert_eq!(e5.format(), Format::TarZstd);
    }
}
