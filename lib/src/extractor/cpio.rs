use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::extractor::{Entries, Entry, ToteExtractor};

/// CPIO format extractor implementation.
///
///
pub(super) struct Extractor {}

impl ToteExtractor for Extractor {
    fn list(&self, target: PathBuf) -> Result<Entries> {
        log::info!("listing CPIO archive: {target:?}");
        let mut file = super::buf_open(&target).map(cpio::Archive::new)?;
        let mut entries: Vec<Entry> = vec![];
        let mut errs = vec![];
        loop {
            let entry = file.read_entry().map_err(crate::Error::IO)?;
            match entry {
                Some(entry) => {
                    if entry.metadata.is_file() {
                        match entry.path.to_path() {
                            Ok(p) => entries.push(create_new_entry(&p, &entry.metadata)),
                            Err(e) => errs.push(crate::Error::IO(e)),
                        };
                    }
                }
                None => break,
            }
        }
        crate::Error::error_or_else(|| Entries::new(target, entries), errs)
    }

    fn perform(&self, target: PathBuf, base: PathBuf) -> Result<()> {
        log::info!("extracting CPIO archive: {target:?}");
        let mut file = super::buf_open(&target).map(cpio::Archive::new)?;
        let mut errs = vec![];
        loop {
            let r = file.read_entry();
            match r {
                Ok(Some(entry)) => {
                    if !entry.metadata.is_file() {
                        continue;
                    }
                    match prepare_write(&entry, &base) {
                        Ok(dest_path) => write_to(entry, &dest_path, &mut errs),
                        Err(e) => errs.push(e),
                    }
                }
                Ok(None) => break,
                Err(e) => errs.push(crate::Error::IO(e)),
            }
        }
        crate::Error::error_or((), errs)
    }
}

fn prepare_write(entry: &cpio::Entry<BufReader<File>>, base: &Path) -> Result<PathBuf> {
    let path = entry.path.to_path()?;
    let dest_path = super::safe_join(base, &path)?;
    log::info!(
        "extracting {} ({} bytes) to {dest_path:?}",
        entry.path,
        entry.metadata.size()
    );
    super::create_parent_dir_all(&dest_path)?;
    Ok(dest_path)
}

fn write_to(
    mut entry: cpio::Entry<BufReader<File>>,
    dest_path: &Path,
    errs: &mut Vec<crate::Error>,
) {
    match std::fs::File::create(dest_path) {
        Ok(mut dest_file) => {
            if let Err(e) = std::io::copy(&mut entry.reader, &mut dest_file) {
                errs.push(crate::Error::IO(e));
            }
        }
        Err(e) => {
            log::error!("failed to create file {dest_path:?}: {e}");
            errs.push(crate::Error::IO(e));
        }
    }
}

fn create_new_entry(path: &Path, entry: &cpio::Metadata) -> Entry {
    let timestamp = entry.mtime();
    let ndt = chrono::DateTime::from_timestamp(timestamp as i64, 0).map(|d| d.naive_utc());
    Entry::builder()
        .name(path.to_string_lossy().to_string())
        .original_size(entry.size())
        .unix_mode(entry.mode())
        .date(ndt)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("../testdata/test.cpio");
        let extractor = Extractor {};
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.first(), Some("./Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("./LICENSE".to_string()).as_ref());
                assert_eq!(r.get(2), Some("./build.rs".to_string()).as_ref());
                assert_eq!(r.get(3), Some("./README.md".to_string()).as_ref());
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.cpio");
        let opts = crate::ExtractConfig::builder()
            .dest("results/cpio")
            .use_archive_name_dir(true)
            .overwrite(true)
            .build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/cpio/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/cpio")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }
}
