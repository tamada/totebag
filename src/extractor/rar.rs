use std::fmt::Debug;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::format::Format;
use crate::cli::{Result, ToatError};
use crate::extractor::{ExtractorOpts, Extractor};

pub(super) struct RarExtractor {
}

impl Extractor for RarExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let mut r = Vec::<String>::new();
        for entry in unrar::Archive::new(&archive_file).open_for_listing().unwrap() {
            let header = entry.unwrap();
            let name = header.filename.to_str().unwrap();
            r.push(name.to_string())
        };
        Ok(r)
    }

    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let archive = unrar::Archive::new(&archive_file);
        let mut file = archive.open_for_processing().unwrap();
        while let Some(header) = file.read_header().unwrap() {
            let name = header.entry().filename.to_str().unwrap();
            let dest = opts.destination(&archive_file).join(PathBuf::from(name));
            file = if header.entry().is_file() {
                opts.v.verbose(format!("extracting {} ({} bytes)", name, header.entry().unpacked_size));
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
        let extractor = RarExtractor{};
        let file = PathBuf::from("testdata/test.rar");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 18);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }
}