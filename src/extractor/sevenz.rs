use std::fs::File;
use std::path::PathBuf;

use crate::{Result, ToteError};
use chrono::DateTime;
use sevenz_rust::{Archive, BlockDecoder, Password, SevenZArchiveEntry};

use crate::extractor::{Entry, PathUtils, ToteExtractor};

pub(super) struct SevenZExtractor {}

impl ToteExtractor for SevenZExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Vec<Entry>> {
        let mut reader = File::open(archive_file).unwrap();
        let len = reader.metadata().unwrap().len();
        match Archive::read(&mut reader, len, Password::empty().as_ref()) {
            Ok(archive) => {
                let mut r = vec![];
                for entry in &archive.files {
                    r.push(convert(entry));
                }
                Ok(r)
            }
            Err(e) => Err(ToteError::Fatal(Box::new(e))),
        }
    }

    fn perform(&self, archive_file: PathBuf, opts: PathUtils) -> Result<()> {
        let file = match File::open(archive_file) {
            Ok(file) => file,
            Err(e) => return Err(ToteError::IO(e)),
        };
        extract(&file, opts)
    }
}

fn convert(e: &SevenZArchiveEntry) -> Entry {
    let name = e.name().to_string();
    let compressed_size = e.compressed_size;
    let uncompressed_size = e.size;
    let mtime = e.last_modified_date.to_unix_time();
    let dt = DateTime::from_timestamp(mtime, 0);
    Entry::new(
        name,
        Some(compressed_size),
        Some(uncompressed_size),
        None,
        dt.map(|dt| dt.naive_local()),
    )
}

fn extract(mut file: &File, opts: PathUtils) -> Result<()> {
    let len = file.metadata().unwrap().len();
    let password = Password::empty();
    let archive = match Archive::read(&mut file, len, password.as_ref()) {
        Ok(reader) => reader,
        Err(e) => return Err(ToteError::Fatal(Box::new(e))),
    };
    let folder_count = archive.folders.len();
    for findex in 0..folder_count {
        let folder_decoder = BlockDecoder::new(findex, &archive, password.as_slice(), &mut file);
        if let Err(e) = folder_decoder.for_each_entries(&mut |entry, reader| {
            let d = opts.destination(PathBuf::from(entry.name.clone())).unwrap();
            sevenz_rust::default_entry_extract_fn(entry, reader, &d)
        }) {
            return Err(ToteError::Fatal(Box::new(e)));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::Extractor;

    #[test]
    fn test_list() {
        let file = PathBuf::from("testdata/test.7z");
        let extractor = SevenZExtractor {};
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 21);
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
        let archive_file = PathBuf::from("testdata/test.7z");
        let opts = Extractor::builder()
            .archive_file(archive_file)
            .destination("results/sevenz")
            .build();
        match opts.perform() {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/sevenz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/sevenz")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
