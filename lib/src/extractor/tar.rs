use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use crate::{Result, Error};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::extractor::{Entry as ToteEntry, Entries, ToteExtractor};

/// TAR format extractor implementation.
pub(super) struct Extractor {}

/// TAR+GZIP format extractor implementation.
pub(super) struct GzExtractor {}

/// TAR+BZIP2 format extractor implementation.
pub(super) struct Bz2Extractor {}

/// TAR+XZ format extractor implementation.
pub(super) struct XzExtractor {}

/// TAR+ZSTD format extractor implementation.
pub(super) struct ZstdExtractor {}

impl ToteExtractor for Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, |f| f)
            .and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, |f| f)
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for GzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, flate2::read::GzDecoder::new)
            .and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, flate2::read::GzDecoder::new)
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for Bz2Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, bzip2::read::BzDecoder::new)
            .and_then(|archive| list_tar(archive, archive_file))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, bzip2::read::BzDecoder::new)
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for XzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, XzDecoder::new)
            .and_then(|archive| list_tar(archive, archive_file))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, XzDecoder::new)
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for ZstdExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap())
            .and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap())
            .and_then(|archive| extract_tar(archive, base))
    }
}

fn open_tar_file<F, R: Read>(file: &PathBuf, opener: F) -> Result<Archive<R>>
where
    F: FnOnce(File) -> R,
{
    let file = match File::open(file) {
        Ok(f) => f,
        Err(e) => return Err(Error::IO(e)),
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
        .date(datetime.map(|dt| dt.naive_local()))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tar_file() {
        let file = PathBuf::from("../testdata/test.tar");
        let extractor = Extractor {};
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
        let extractor = Bz2Extractor {};
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
        let extractor = GzExtractor {};
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
        let extractor = ZstdExtractor {};
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
    fn test_list_tar_xz_file() {
        let file = PathBuf::from("../testdata/test.tar.xz");
        let extractor = XzExtractor {};
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
    fn test_extract_tar_archive() {
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

    #[test]
    fn test_extract_targz_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.gz");
        let opts = crate::ExtractConfig::builder().dest("results/targz").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/targz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/targz")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_extract_tarbz2_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.bz2");
        let opts = crate::ExtractConfig::builder().dest("results/tarbz2").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/tarbz2/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarbz2")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_extract_tarxz_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.xz");
        let opts = crate::ExtractConfig::builder().dest("results/tarxz").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/tarxz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarxz")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_extract_tarzstd_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.zst");
        let opts = crate::ExtractConfig::builder().dest("results/tarzstd").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/tarzstd/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarzstd")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
