use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use crate::{Result, ToteError};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extractor::{Entry as ToteEntry, Entries, ToteExtractor};

/// TAR format extractor implementation.
pub(super) struct TarExtractor {}

/// TAR+GZIP format extractor implementation.
pub(super) struct TarGzExtractor {}

/// TAR+BZIP2 format extractor implementation.
pub(super) struct TarBz2Extractor {}

/// TAR+XZ format extractor implementation.
pub(super) struct TarXzExtractor {}

/// TAR+ZSTD format extractor implementation.
pub(super) struct TarZstdExtractor {}

impl ToteExtractor for TarExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        match open_tar_file(&archive_file, |f| f) {
            Ok(archive) => list_tar(archive, archive_file),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        match open_tar_file(&archive_file, |f| f) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, base),
        }
    }
}

impl ToteExtractor for TarGzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        match open_tar_file(&archive_file, flate2::read::GzDecoder::new) {
            Ok(archive) => list_tar(archive, archive_file),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        match open_tar_file(&archive_file, flate2::read::GzDecoder::new) {
            Ok(archive) => extract_tar(archive, base),
            Err(e) => Err(e),
        }
    }
}

impl ToteExtractor for TarBz2Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        match open_tar_file(&archive_file, bzip2::read::BzDecoder::new) {
            Ok(archive) => list_tar(archive, archive_file),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        match open_tar_file(&archive_file, bzip2::read::BzDecoder::new) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, base),
        }
    }
}

impl ToteExtractor for TarXzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        match open_tar_file(&archive_file, XzDecoder::new) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive, archive_file),
        }
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        match open_tar_file(&archive_file, XzDecoder::new) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, base),
        }
    }
}

impl ToteExtractor for TarZstdExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        match open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive, archive_file),
        }
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        match open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, base),
        }
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

fn extract_tar<R: Read>(mut archive: tar::Archive<R>, base: PathBuf) -> Result<()> {
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.header().path().unwrap();
        let p = path.clone().to_path_buf();
        if is_filename_mac_finder_file(p.to_path_buf()) {
            continue;
        }
        let size = entry.header().size().unwrap();
        log::info!("extracting {path:?} ({size} bytes)");

        let dest = base.join(&path);
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

fn list_tar<R: Read>(mut archive: tar::Archive<R>, path: PathBuf) -> Result<Entries> {
    let mut result = vec![];
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        result.push(tar_entry_to_entry(entry));
    }
    Ok(Entries::new(path, result))
}

fn tar_entry_to_entry<R: Read>(e: tar::Entry<R>) -> ToteEntry {
    let header = e.header();
    let path = header.path().unwrap().to_str().unwrap().to_string();
    let size = header.size();
    let mode = header.mode().unwrap();
    let mtime = header.mtime().unwrap();
    let datetime = chrono::DateTime::from_timestamp_millis(mtime as i64);
    ToteEntry::builder()
        .name(path)
        .original_size(size.unwrap())
        .unix_mode(mode)
        .date(datetime.map(|dt| dt.naive_local()).unwrap())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tar_file() {
        let file = PathBuf::from("../testdata/test.tar");
        let extractor = TarExtractor {};
        match extractor.list(file) {
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
    fn test_list_tarbz2_file() {
        let file = PathBuf::from("../testdata/test.tar.bz2");
        let extractor = TarBz2Extractor {};
        match extractor.list(file) {
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
        let file = PathBuf::from("../testdata/test.tar.gz");
        let extractor = TarGzExtractor {};
        match extractor.list(file) {
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
        let file = PathBuf::from("../testdata/test.tar.zst");
        let extractor = TarZstdExtractor {};
        match extractor.list(file) {
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
        let archive_file = PathBuf::from("../testdata/test.tar");
        let opts = crate::ExtractConfig::builder().dest("results/tar").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/tar/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tar")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
