use std::fs::create_dir_all;
use std::path::PathBuf;

use chrono::DateTime;
use unrar::FileHeader;

use crate::Result;

use crate::extractor::{Entry, Extractor, ToteExtractor};
use crate::format::Format;

pub(super) struct RarExtractor {
    target: PathBuf,
}

impl RarExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

impl ToteExtractor for RarExtractor {
    fn list(&self) -> Result<Vec<Entry>> {
        let mut r = vec![];
        for entry in unrar::Archive::new(&self.target)
            .open_for_listing()
            .unwrap()
        {
            let header = entry.unwrap();
            r.push(convert(header));
        }
        Ok(r)
    }

    fn perform(&self, opts: &Extractor) -> Result<()> {
        let archive = unrar::Archive::new(&self.target);
        let mut file = archive.open_for_processing().unwrap();
        while let Some(header) = file.read_header().unwrap() {
            let name = header.entry().filename.to_str().unwrap();
            let dest = opts.base_dir().join(PathBuf::from(name));
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

fn convert(fh: FileHeader) -> Entry {
    let name = fh.filename.to_str().unwrap();
    let uncompressed_size = fh.unpacked_size;
    let mtime = fh.file_time as i64;
    let dt = DateTime::from_timestamp(mtime, 0);
    Entry::new(
        name.to_string(),
        None,
        Some(uncompressed_size),
        None,
        dt.map(|dt| dt.naive_local()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("testdata/test.rar");
        let extractor = RarExtractor::new(file);
        match extractor.list() {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
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
        let archive_file = PathBuf::from("testdata/test.rar");
        let opts = Extractor::builder()
            .archive_file(archive_file.clone())
            .destination(PathBuf::from("results/rar"))
            .use_archive_name_dir(true)
            .build();
        match opts.perform() {
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
        let e = RarExtractor::new(PathBuf::from("testdata/test.rar"));
        assert_eq!(e.format(), Format::Rar);
    }
}
