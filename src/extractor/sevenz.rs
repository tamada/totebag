use std::fs::File;
use std::path::PathBuf;

use sevenz_rust::{Archive, BlockDecoder, Password};

use crate::extractor::Extractor;
use crate::format::Format;
use crate::cli::{Result, ToteError};

use super::ExtractorOpts;

pub(super) struct SevenZExtractor {
}

impl Extractor for SevenZExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let mut reader = File::open(archive_file).unwrap();
        let len = reader.metadata().unwrap().len();
        match Archive::read(&mut reader,len, Password::empty().as_ref()) {
            Ok(archive) => {
                let mut r = Vec::<String>::new();
                for entry in &archive.files {
                    r.push(entry.name.clone())
                }
                Ok(r)
            },
            Err(e) => Err(ToteError::Fatal(Box::new(e))),
        }
    }
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let mut file = match File::open(&archive_file) {
            Ok(file) => {
                file
            },
            Err(e) => return Err(ToteError::IO(e)),
        };
        extract(&mut file, archive_file, opts)
    }
    fn format(&self) -> Format {
        Format::SevenZ
    }
}

fn extract(mut file: &File, path: PathBuf, opts: &ExtractorOpts) -> Result<()> {
    let len = file.metadata().unwrap().len();
    let password = Password::empty();
    let archive = match Archive::read(&mut file, len, password.as_ref()) {
        Ok(reader) => {
            reader
        },
        Err(e) => return Err(ToteError::Fatal(Box::new(e))),
    };
    let folder_count = archive.folders.len();
    for findex in 0..folder_count {
        let folder_decoder = BlockDecoder::new(findex, &archive, password.as_slice(), &mut file);
        if let Err(e) = folder_decoder.for_each_entries(&mut |entry, reader| {
            let dest = opts.destination(&path).join(entry.name.clone());
            sevenz_rust::default_entry_extract_fn(entry, reader, &dest)
        }) {
            return Err(ToteError::Fatal(Box::new(e)))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verboser::create_verboser;

    #[test]
    fn test_list() {
        let extractor = SevenZExtractor{};
        let file = PathBuf::from("testdata/test.7z");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 21);
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
        let e = SevenZExtractor{};
        let file = PathBuf::from("testdata/test.7z");
        let opts = ExtractorOpts {
            dest: PathBuf::from("results/sevenz"),
            use_archive_name_dir: false,
            overwrite: true,
            v: create_verboser(false),
        };
        match e.perform(file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/sevenz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/sevenz")).unwrap();
            },
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let e = SevenZExtractor{};
        assert_eq!(e.format(), Format::SevenZ);
    }
}