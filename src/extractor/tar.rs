use std::fs::create_dir_all;
use std::io::Read;
use std::{fs::File, path::PathBuf};

use xz2::read::XzDecoder;

use crate::cli::Result;
use crate::extractor::{Extractor, ExtractorOpts};
use crate::format::Format;

pub(super) struct TarExtractor {
}
pub(super) struct TarGzExtractor {
}
pub(super) struct TarBz2Extractor {
}
pub(super) struct TarXzExtractor {
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

impl Extractor for TarXzExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let file = File::open(archive_file).unwrap();
        let tarxz = XzDecoder::new(file);
        let mut archive = tar::Archive::new(tarxz);
        list_tar(&mut archive)
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let file = File::open(&archive_file).unwrap();
        let tarxz = XzDecoder::new(file);
        let mut archive = tar::Archive::new(tarxz);
        extract_tar(&mut archive, archive_file, opts)
    }
    fn format(&self) -> Format {
        Format::TarXz
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
    use crate::verboser::create_verboser;

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
    fn test_extract_archive() {
        let e = TarExtractor{};
        let file = PathBuf::from("testdata/test.tar");
        let opts = ExtractorOpts {
            dest: PathBuf::from("results/tar"),
            use_archive_name_dir: false,
            overwrite: true,
            v: create_verboser(false),
        };
        match e.perform(file, &opts) {
            Ok(_) =>  {
                assert!(true);
                assert!(PathBuf::from("results/tar/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/tar")).unwrap();
            },
            Err(_) => assert!(false),
        };
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

    #[test]
    fn test_format() {
        let e1 = TarExtractor{};
        assert_eq!(e1.format(), Format::Tar);

        let e2 = TarGzExtractor{};
        assert_eq!(e2.format(), Format::TarGz);

        let e3 = TarBz2Extractor{};
        assert_eq!(e3.format(), Format::TarBz2);
    }
}