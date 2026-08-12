use std::fs::File;
use std::path::PathBuf;

use crate::{Error, Result};
use chrono::DateTime;
use sevenz_rust2::{Archive, ArchiveEntry, BlockDecoder, Password};

use crate::extractor::{Entries, Entry, ToteExtractor};

/// Number of threads `BlockDecoder` may use. Kept at 1 so that extraction stays
/// deterministic and does not spawn threads behind the caller's back.
const THREAD_COUNT: u32 = 1;

/// 7-Zip format extractor implementation.
///
/// This extractor handles 7z archive files.
pub(super) struct Extractor {}

impl ToteExtractor for Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        let mut reader = File::open(&archive_file).map_err(Error::IO)?;
        match Archive::read(&mut reader, &Password::empty()) {
            Ok(archive) => {
                let r = archive.files.iter().map(convert).collect();
                Ok(Entries::new(archive_file, r))
            }
            Err(e) => Err(Error::Extractor(e.to_string())),
        }
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        let file = File::open(archive_file).map_err(Error::IO)?;
        extract(&file, base)
    }
}

fn convert(e: &ArchiveEntry) -> Entry {
    let mtime = std::time::SystemTime::from(e.last_modified_date)
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0));
    Entry::builder()
        .name(e.name.clone())
        .compressed_size(e.compressed_size)
        .original_size(e.size)
        .date(mtime.map(|dt| dt.naive_local()))
        .build()
}

fn extract(mut file: &File, base: PathBuf) -> Result<()> {
    let password = Password::empty();
    let archive = match Archive::read(&mut file, &password) {
        Ok(reader) => reader,
        Err(e) => return Err(Error::Fatal(Box::new(e))),
    };
    for block_index in 0..archive.blocks.len() {
        let block_decoder =
            BlockDecoder::new(THREAD_COUNT, block_index, &archive, &password, &mut file);
        if let Err(e) = block_decoder.for_each_entries(&mut |entry, reader| {
            let d = base.join(&entry.name);
            sevenz_rust2::default_entry_extract_fn(entry, reader, &d)
        }) {
            return Err(Error::Fatal(Box::new(e)));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let file = PathBuf::from("../testdata/test.7z");
        let extractor = Extractor {};
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 21);
                assert_eq!(r.first(), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.7z");
        let opts = crate::ExtractConfig::builder()
            .dest("results/sevenz")
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/sevenz/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/sevenz")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }
}
