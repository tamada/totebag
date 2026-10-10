use std::path::PathBuf;

use crate::extractor::{Entries, Entry, ToteExtractor};
use crate::{Error, Result};
use chrono::{DateTime, Utc};

/// RAR format extractor implementation.
///
/// This extractor handles RAR archive files.
pub(super) struct Extractor {}

impl ToteExtractor for Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        let mut r = vec![];
        let archive = rars::ArchiveReader::read_path(&archive_file)
            .map_err(|e| Error::Extractor(e.to_string()))?;
        for entry in archive.members() {
            r.push(convert(entry));
        }
        Ok(Entries::new(archive_file, r))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        let archive = rars::ArchiveReader::read_path(&archive_file)
            .map_err(|e| Error::Extractor(e.to_string()))?;
        let mut errs = vec![];
        let r = archive.extract_to(None, |meta| {
            let name = match rars::entry_relative_path(&meta.name) {
                Ok(rel) => rel,
                Err(e) => {
                    errs.push(Error::Extractor(e.to_string()));
                    return Ok(Box::new(std::io::sink()));
                }
            };
            let dest = base.join(&name);
            let r = if !meta.is_directory {
                log::info!("extracting {}", name.display());
                if let Err(e) = super::create_parent_dir_all(&dest) {
                    Err(e)
                } else {
                    match std::fs::File::create(dest) {
                        Ok(file) => Ok(Some(file)),
                        Err(e) => Err(Error::IO(e)),
                    }
                }
            } else {
                Ok(None)
            };
            match r {
                Ok(None) => Ok(Box::new(std::io::sink()) as Box<dyn std::io::Write>),
                Ok(Some(file)) => Ok(Box::new(file)),
                Err(e) => {
                    errs.push(e);
                    Ok(Box::new(std::io::sink()) as Box<dyn std::io::Write>)
                }
            }
        });
        match r {
            Ok(()) => Error::error_or((), errs),
            Err(e) => {
                errs.push(Error::Extractor(e.to_string()));
                Error::error_or((), errs)
            }
        }
    }
}

fn convert(member: rars::ArchiveMember) -> Entry {
    let mtime = member
        .meta
        .modification_time()
        .map(|s| DateTime::<Utc>::from(s).naive_local());
    Entry::builder()
        .name(member.meta.name_lossy())
        .compressed_size(member.meta.packed_size)
        .original_size(member.meta.unpacked_size)
        .date(mtime)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let extractor = Extractor {};
        let file = PathBuf::from("../testdata/test.rar");
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 18);
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
        let archive_file = PathBuf::from("../testdata/test.rar");
        let opts = crate::ExtractConfig::builder()
            .dest(PathBuf::from("results/rar"))
            .use_archive_name_dir(true)
            .overwrite(true)
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/rar/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/rar")).unwrap();
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        };
    }
}
