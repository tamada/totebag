use std::fs::File;
use std::path::PathBuf;

use crate::{Result, ToteError};
use sevenz_rust::{Archive, BlockDecoder, Password};

use crate::extractor::ToteExtractor as Extractor;
use crate::format::Format;

use super::ExtractorOpts;

pub(super) struct SevenZExtractor {}

impl Extractor for SevenZExtractor {
    fn list(&self, archive_file: &PathBuf) -> Result<Vec<String>> {
        let mut reader = File::open(archive_file).unwrap();
        let len = reader.metadata().unwrap().len();
        match Archive::read(&mut reader, len, Password::empty().as_ref()) {
            Ok(archive) => {
                let mut r = Vec::<String>::new();
                for entry in &archive.files {
                    r.push(entry.name.clone())
                }
                Ok(r)
            }
            Err(e) => Err(ToteError::Fatal(Box::new(e))),
        }
    }
    fn perform(&self, archive_file: &PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let mut file = match File::open(&archive_file) {
            Ok(file) => file,
            Err(e) => return Err(ToteError::IO(e)),
        };
        extract(archive_file, &mut file, opts)
    }
    fn format(&self) -> Format {
        Format::SevenZ
    }
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
        let extractor = SevenZExtractor {};
        let file = PathBuf::from("testdata/test.7z");
        match extractor.list(&file) {
            Ok(r) => {
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
        let e = SevenZExtractor {};
        let archive_file = PathBuf::from("testdata/test.7z");
        let dest = PathBuf::from("results/sevenz");
        let opts = ExtractorOpts::new_with_opts(Some(dest), false, true);
        match e.perform(&archive_file, &opts) {
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
        let e = SevenZExtractor {};
        assert_eq!(e.format(), Format::SevenZ);
    }
}
