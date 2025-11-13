use std::fs::create_dir_all;
use std::path::PathBuf;

use chrono::DateTime;
use unrar::FileHeader;

use crate::{Result, ToteError};

use crate::extractor::{Entry, ToteExtractor};

pub(super) struct RarExtractor {}

impl ToteExtractor for RarExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Vec<Entry>> {
        let mut r = vec![];
        for entry in unrar::Archive::new(&archive_file)
            .open_for_listing()
            .unwrap()
        {
            let header = entry.unwrap();
            r.push(convert(header));
        }
        Ok(r)
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        let archive = unrar::Archive::new(&archive_file);
        let mut file = archive.open_for_processing().unwrap();
        while let Some(header) = file.read_header().unwrap() {
            let name = header.entry().filename.to_str().unwrap();
            let dest = base.join(name);
            file = if header.entry().is_file() {
                log::info!(
                    "extracting {} ({} bytes)",
                    name,
                    header.entry().unpacked_size
                );
                if let Err(e) = create_dir_all(dest.parent().unwrap()) {
                    return Err(ToteError::IO(e));
                }
                header.extract_to(&dest).unwrap()
            } else {
                header.skip().unwrap()
            }
        }
        Ok(())
    }
}

fn convert(fh: FileHeader) -> Entry {
    let name = fh.filename.to_str().unwrap();
    let uncompressed_size = fh.unpacked_size;
    let mtime = fh.file_time as i64;
    let dt = DateTime::from_timestamp(mtime, 0);
    Entry::builder()
        .name(name)
        .original_size(uncompressed_size)
        .date(dt.map(|dt| dt.naive_local()).unwrap())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let extractor = RarExtractor {};
        let file = PathBuf::from("../testdata/test.rar");
        match extractor.list(file) {
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
        let archive_file = PathBuf::from("../testdata/test.rar");
        let opts = crate::ExtractConfig::builder()
            .dest(PathBuf::from("results/rar"))
            .use_archive_name_dir(true)
            .overwrite(true)
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/rar/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/rar")).unwrap();
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        };
    }
}
