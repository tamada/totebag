use std::fs::create_dir_all;
use std::io::Read;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::{Error, Result};
use lzma_rust2::XzReader;
use tar::Archive;

use crate::extractor::{Entries, Entry as ToteEntry, ToteExtractor};

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
        open_tar_file(&archive_file, Ok).and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, Ok).and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for GzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, |f| Ok(flate2::read::GzDecoder::new(f)))
            .and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, |f| Ok(flate2::read::GzDecoder::new(f)))
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for Bz2Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, |f| Ok(bzip2::read::BzDecoder::new(f)))
            .and_then(|archive| list_tar(archive, archive_file))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, |f| Ok(bzip2::read::BzDecoder::new(f)))
            .and_then(|archive| extract_tar(archive, base))
    }
}

impl ToteExtractor for XzExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, new_xz_decoder)
            .and_then(|archive| list_tar(archive, archive_file))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, new_xz_decoder).and_then(|archive| extract_tar(archive, base))
    }
}

/// `.tar.xz` files produced by `xz(1)` may hold several concatenated streams,
/// so multi-stream input is allowed.
fn new_xz_decoder(f: File) -> Result<XzReader<File>> {
    Ok(XzReader::new(f, true))
}

impl ToteExtractor for ZstdExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        open_tar_file(&archive_file, zstd::new_decoder)
            .and_then(|archive| list_tar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        open_tar_file(&archive_file, zstd::new_decoder)
            .and_then(|archive| extract_tar(archive, base))
    }
}

/// The zstd decoder backend. `ruzstd` implements the whole decompression side of
/// the specification, so the pure Rust path is used unless `zstd-native` asks for
/// the C library.
#[cfg(not(feature = "zstd-native"))]
mod zstd {
    use crate::{Error, Result};
    use std::fs::File;
    use std::io::BufReader;

    type Decoder =
        ruzstd::decoding::StreamingDecoder<BufReader<File>, ruzstd::decoding::FrameDecoder>;

    pub(super) fn new_decoder(f: File) -> Result<Decoder> {
        ruzstd::decoding::StreamingDecoder::new(BufReader::new(f))
            .map_err(|e| Error::Extractor(e.to_string()))
    }
}

#[cfg(feature = "zstd-native")]
mod zstd {
    use crate::{Error, Result};
    use std::fs::File;
    use std::io::BufReader;

    type Decoder = ::zstd::Decoder<'static, BufReader<File>>;

    pub(super) fn new_decoder(f: File) -> Result<Decoder> {
        ::zstd::Decoder::new(f).map_err(Error::IO)
    }
}

fn open_tar_file<F, R: Read>(file: &PathBuf, opener: F) -> Result<Archive<R>>
where
    F: FnOnce(File) -> Result<R>,
{
    let file = File::open(file).map_err(Error::IO)?;
    opener(file).map(Archive::new)
}

fn extract_tar<R: Read>(mut archive: tar::Archive<R>, base: PathBuf) -> Result<()> {
    for entry in archive.entries().map_err(Error::IO)? {
        let mut entry = entry.map_err(Error::IO)?;
        let path = entry.header().path().map_err(Error::IO)?.into_owned();
        if is_filename_mac_finder_file(&path) {
            continue;
        }
        let size = entry.header().size().map_err(Error::IO)?;
        log::info!("extracting {path:?} ({size} bytes)");

        let dest = base.join(&path);
        if entry.header().entry_type().is_file() {
            if let Some(parent) = dest.parent() {
                create_dir_all(parent).map_err(Error::IO)?;
            }
            entry.unpack(dest).map_err(Error::IO)?;
        }
    }
    Ok(())
}

fn is_filename_mac_finder_file(path: &Path) -> bool {
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name == ".DS_Store" || name.starts_with("._"),
        None => false,
    }
}

fn list_tar<R: Read>(mut archive: tar::Archive<R>, path: PathBuf) -> Result<Entries> {
    let mut result = vec![];
    for entry in archive.entries().map_err(Error::IO)? {
        let entry = entry.map_err(Error::IO)?;
        result.push(tar_entry_to_entry(entry));
    }
    Ok(Entries::new(path, result))
}

fn tar_entry_to_entry<R: Read>(e: tar::Entry<R>) -> ToteEntry {
    let header = e.header();
    let name = header
        .path()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    // The tar header records mtime in *seconds* since the epoch.
    let datetime = header
        .mtime()
        .ok()
        .and_then(|secs| chrono::DateTime::from_timestamp(secs as i64, 0));
    ToteEntry::builder()
        .name(name)
        .original_size(header.size().unwrap_or(0))
        .unix_mode(header.mode().unwrap_or(0o644))
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
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
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
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
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
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
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
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
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
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_extract_tar_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar");
        let opts = crate::ExtractConfig::builder().dest("results/tar").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/tar/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tar")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }

    #[test]
    fn test_extract_targz_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.gz");
        let opts = crate::ExtractConfig::builder()
            .dest("results/targz")
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/targz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/targz")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }

    #[test]
    fn test_extract_tarbz2_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.bz2");
        let opts = crate::ExtractConfig::builder()
            .dest("results/tarbz2")
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/tarbz2/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarbz2")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }

    #[test]
    fn test_extract_tarxz_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.xz");
        let opts = crate::ExtractConfig::builder()
            .dest("results/tarxz")
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/tarxz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarxz")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }

    #[test]
    fn test_extract_tarzstd_archive() {
        let archive_file = PathBuf::from("../testdata/test.tar.zst");
        let opts = crate::ExtractConfig::builder()
            .dest("results/tarzstd")
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/tarzstd/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tarzstd")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }
}
