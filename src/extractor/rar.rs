use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::Result;

use crate::extractor::{ExtractorOpts, ToteExtractor as Extractor};
use crate::format::Format;

pub(super) struct RarExtractor {}

impl Extractor for RarExtractor {
    fn list(&self, archive_file: &PathBuf) -> Result<Vec<String>> {
        let mut r = Vec::<String>::new();
        for entry in unrar::Archive::new(&archive_file)
            .open_for_listing()
            .unwrap()
        {
            let header = entry.unwrap();
            let name = header.filename.to_str().unwrap();
            r.push(name.to_string())
        }
        Ok(r)
    }

    fn perform(&self, archive_file: &PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let archive = unrar::Archive::new(&archive_file);
        let mut file = archive.open_for_processing().unwrap();
        while let Some(header) = file.read_header().unwrap() {
            let name = header.entry().filename.to_str().unwrap();
            let dest = opts.base_dir(archive_file).join(PathBuf::from(name));
            file = if header.entry().is_file() {
                log::info!(
                    "extracting {} ({} bytes)",
                    name,
                    header.entry().unpacked_size
                );
                create_dir_all(dest.parent().unwrap()).unwrap();
                header.extract_to(&dest).unwrap()
            } else {
                header.skip().unwrap()
            }
        }
        Ok(())
    }

    fn format(&self) -> Format {
        Format::Rar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let extractor = RarExtractor {};
        let file = PathBuf::from("testdata/test.rar");
        match extractor.list(&file) {
            Ok(r) => {
                assert_eq!(r.len(), 18);
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
        let e = RarExtractor {};
        let archive_file = PathBuf::from("testdata/test.rar");
        let dest = PathBuf::from("results/rar");
        let opts = ExtractorOpts::new_with_opts(Some(dest), true, true);
        match e.perform(&archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/rar/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/rar")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let extractor = RarExtractor {};
        assert_eq!(extractor.format(), Format::Rar);
    }
}
