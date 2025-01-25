use std::fs::File;
use std::path::PathBuf;

use crate::{Result, ToteError};
use chrono::DateTime;
use sevenz_rust::{Archive, BlockDecoder, Password, SevenZArchiveEntry};

use crate::extractor::{Entry, Extractor, ExtractorOpts};
use crate::format::Format;

pub(super) struct SevenZExtractor {
    target: PathBuf,
}

impl SevenZExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

impl Extractor for SevenZExtractor {
    fn list(&self) -> Result<Vec<Entry>> {
        let mut reader = File::open(&self.target).unwrap();
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

    fn perform(&self, opts: &ExtractorOpts) -> Result<()> {
        let mut file = match File::open(&self.target) {
            Ok(file) => file,
            Err(e) => return Err(ToteError::IO(e)),
        };
        extract(&self.target, &mut file, opts)
    }
    fn format(&self) -> Format {
        Format::SevenZ
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

fn extract(archive_file: &PathBuf, mut file: &File, opts: &ExtractorOpts) -> Result<()> {
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
            let d = opts.base_dir(archive_file).join(&entry.name);
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

    #[test]
    fn test_list() {
        let file = PathBuf::from("testdata/test.7z");
        let extractor = SevenZExtractor::new(file);
        match extractor.list() {
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
        let e = SevenZExtractor::new(archive_file.clone());
        let dest = PathBuf::from("results/sevenz");
        let opts = ExtractorOpts::new_with_opts(Some(dest), false, true);
        match e.perform(&opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/sevenz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/sevenz")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let e = SevenZExtractor::new(PathBuf::from("testdata/test.7z"));
        assert_eq!(e.format(), Format::SevenZ);
    }
}
