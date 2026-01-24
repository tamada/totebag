use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf};

use cab::{Cabinet, FileEntry};

use crate::extractor::{Entries, Entry, ToteExtractor};
use crate::{Result, ToteError};

/// CAB (Cabinet) format extractor implementation.
///
/// This extractor handles Microsoft Cabinet archive files.
pub(super) struct CabExtractor {}

impl ToteExtractor for CabExtractor {
    fn list(&self, target: PathBuf) -> Result<Entries> {
        list_impl(&target, convert)
            .map(|r| Entries::new(target, r))
    }

    fn perform(&self, target: PathBuf, base: PathBuf) -> Result<()> {
        let list = list_impl(&target, |file| {
            (file.name().to_string(), file.uncompressed_size())
        })?;
        let mut errs = vec![];
        let mut cabinet = open_cabinet(&target)?;
        for file in list {
            if let Err(e) = write_file_impl(&mut cabinet, file, &base) {
                errs.push(e);
            }
        }
        ToteError::error_or((), errs)
    }
}

fn write_file_impl(cabinet: &mut Cabinet<File>, file: (String, u32), base: &Path) -> Result<()> {
    let file_name = file.0.clone();
    let dest_file = base.join(&file_name);
    log::info!("extracting {file_name} ({} bytes)", file.1);
    match create_dir_all(dest_file.parent().unwrap()) {
        Ok(_) => {}
        Err(e) => return Err(ToteError::IO(e)),
    }
    match File::create(dest_file) {
        Ok(mut dest) => {
            let mut file_from = cabinet.read_file(&file_name).unwrap();
            match std::io::copy(&mut file_from, &mut dest) {
                Ok(_) => Ok(()),
                Err(e) => Err(ToteError::IO(e)),
            }
        }
        Err(e) => Err(ToteError::IO(e)),
    }
}

fn open_cabinet(archive_file: &PathBuf) -> Result<Cabinet<File>> {
    let cab_file = match File::open(archive_file) {
        Ok(f) => f,
        Err(e) => return Err(ToteError::IO(e)),
    };
    match Cabinet::new(cab_file) {
        Ok(c) => Ok(c),
        Err(e) => Err(ToteError::IO(e)),
    }
}

fn list_impl<F, T>(archive_file: &PathBuf, mapper: F) -> Result<Vec<T>>
where
    F: Fn(&cab::FileEntry) -> T,
{
    let cabinet = open_cabinet(archive_file)?;
    let mut result = vec![];
    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            result.push(mapper(file));
        }
    }
    Ok(result)
}

fn convert(f: &FileEntry) -> Entry {
    let name = f.name().to_string();
    let uncompressed_size = f.uncompressed_size();
    let mtime = f.datetime().map(to_naive_datetime);
    Entry::builder()
        .name(name)
        .original_size(uncompressed_size as u64)
        .date(mtime.unwrap())
        .build()
}

fn to_naive_datetime(t: time::PrimitiveDateTime) -> chrono::NaiveDateTime {
    let timestamp = t.assume_utc().unix_timestamp();
    chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap()
        .naive_local()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("../testdata/test.cab");
        let extractor = CabExtractor {};
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(2), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.cab");
        let dest = PathBuf::from("results/cab");
        let opts = crate::ExtractConfig::builder()
            .dest(dest)
            .use_archive_name_dir(true)
            .build();

        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/cab/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/cab")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
