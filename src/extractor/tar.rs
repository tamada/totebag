use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use tar::Archive;
use xz2::read::XzDecoder;

use crate::cli::{Result, ToteError};
use crate::extractor::{Extractor, ExtractorOpts};
use crate::format::Format;

pub(super) struct TarExtractor {}
pub(super) struct TarGzExtractor {}
pub(super) struct TarBz2Extractor {}
pub(super) struct TarXzExtractor {}
pub(super) struct TarZstdExtractor {}

impl Extractor for TarExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        match open_tar_file(&archive_file, |f| f) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        match open_tar_file(&archive_file, |f| f) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, archive_file, opts),
        }
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}

impl Extractor for TarGzExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        match open_tar_file(&archive_file, |f| flate2::read::GzDecoder::new(f)) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        match open_tar_file(&archive_file, |f| flate2::read::GzDecoder::new(f)) {
            Ok(archive) => extract_tar(archive, archive_file, opts),
            Err(e) => Err(e),
        }
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}

impl Extractor for TarBz2Extractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        match open_tar_file(&archive_file, |f| bzip2::read::BzDecoder::new(f)) {
            Ok(archive) => list_tar(archive),
            Err(e) => Err(e),
        }
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        match open_tar_file(&archive_file, |f| bzip2::read::BzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, archive_file, opts),
        }
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}

impl Extractor for TarXzExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        match open_tar_file(&archive_file, |f| XzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive),
        }
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        match open_tar_file(&archive_file, |f| XzDecoder::new(f)) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, archive_file, opts),
        }
    }
    fn format(&self) -> Format {
        Format::TarXz
    }
}

impl Extractor for TarZstdExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        match open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => list_tar(archive),
        }
    }

    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        match open_tar_file(&archive_file, |f| zstd::Decoder::new(f).unwrap()) {
            Err(e) => Err(e),
            Ok(archive) => extract_tar(archive, archive_file, opts),
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

fn extract_tar<R: Read>(
    mut archive: tar::Archive<R>,
    original: PathBuf,
    opts: &ExtractorOpts,
) -> Result<()> {
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.header().path().unwrap();
        let p = path.clone().to_path_buf();
        if is_filename_mac_finder_file(p.to_path_buf()) {
            continue;
        }
        let size = entry.header().size().unwrap();
        opts.v
            .verbose(format!("extracting {:?} ({} bytes)", path, size));

        let dest = opts.destination(&original).join(path);
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

fn list_tar<R: Read>(mut archive: tar::Archive<R>) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        let path = entry.header().path().unwrap();
        result.push(format!("{}", path.to_str().unwrap()));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verboser::create_verboser;

    #[test]
    fn test_list_tar_file() {
        let extractor = TarExtractor {};
        let file = PathBuf::from("testdata/test.tar");
        match extractor.list_archives(file) {
            Ok(r) => {
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
        let e = TarExtractor {};
        let file = PathBuf::from("testdata/test.tar");
        let opts = ExtractorOpts {
            dest: PathBuf::from("results/tar"),
            use_archive_name_dir: false,
            overwrite: true,
            v: create_verboser(false),
        };
        match e.perform(file, &opts) {
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
        let extractor = TarBz2Extractor {};
        let file = PathBuf::from("testdata/test.tar.bz2");
        match extractor.list_archives(file) {
            Ok(r) => {
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
        let extractor = TarGzExtractor {};
        let file = PathBuf::from("testdata/test.tar.gz");
        match extractor.list_archives(file) {
            Ok(r) => {
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
        let extractor = TarZstdExtractor {};
        let file = PathBuf::from("testdata/test.tar.zst");
        match extractor.list_archives(file) {
            Ok(r) => {
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
        let e1 = TarExtractor {};
        assert_eq!(e1.format(), Format::Tar);

        let e2 = TarGzExtractor {};
        assert_eq!(e2.format(), Format::TarGz);

        let e3 = TarBz2Extractor {};
        assert_eq!(e3.format(), Format::TarBz2);
    }
}
