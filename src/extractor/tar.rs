use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use crate::cli::Result;
use crate::extractor::{Extractor, ExtractorOpts};
use crate::format::Format;

pub(super) struct TarExtractor {
}
pub(super) struct TarGzExtractor {
}
pub(super) struct TarBz2Extractor {
}

impl Extractor for TarExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let file = File::open(archive_file).unwrap();
        let mut archive = tar::Archive::new(file);
        list_tar(&mut archive)
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let file = File::open(&archive_file).unwrap();
        let mut archive = tar::Archive::new(file);
        extract_tar(&mut archive, archive_file, opts)
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}

impl Extractor for TarGzExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let file = File::open(archive_file).unwrap();
        let targz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(targz);
        list_tar(&mut archive)
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let file = File::open(&archive_file).unwrap();
        let targz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(targz);
        extract_tar(&mut archive, archive_file, opts)
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}

impl Extractor for TarBz2Extractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let file = File::open(archive_file).unwrap();
        let tarbz2 = bzip2::read::BzDecoder::new(file);
        let mut archive = tar::Archive::new(tarbz2);
        list_tar(&mut archive)
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let file = File::open(&archive_file).unwrap();
        let tarbz2 = bzip2::read::BzDecoder::new(file);
        let mut archive = tar::Archive::new(tarbz2);
        extract_tar(&mut archive, archive_file, opts)
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}

fn extract_tar<R: Read>(archive: &mut tar::Archive<R>, original: PathBuf, opts: &ExtractorOpts) -> Result<()> {
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path = entry.header().path().unwrap();
        let p = path.clone().to_path_buf();
        if is_filename_mac_finder_file(p.to_path_buf()) {
            continue;
        }
        let size = entry.header().size().unwrap();
        opts.v.verbose(format!("extracting {:?} ({} bytes)", path, size));

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

fn list_tar<R: Read>(archive: &mut tar::Archive<R>) -> Result<Vec<String>> {
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

    #[test]
    fn test_list_tar_file() {
        let extractor = TarExtractor{};
        let file = PathBuf::from("testdata/test.tar");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_list_tarbz2_file() {
        let extractor = TarBz2Extractor{};
        let file = PathBuf::from("testdata/test.tar.bz2");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_list_targz_file() {
        let extractor = TarGzExtractor{};
        let file = PathBuf::from("testdata/test.tar.gz");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }
}