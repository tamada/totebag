use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use cab::{Cabinet, FileEntry};

use crate::extractor::Extractor;
use crate::extractor::{Entry, ExtractorOpts};
use crate::{Result, ToteError};

pub(super) struct CabExtractor {
    target: PathBuf,
}

impl CabExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

impl Extractor for CabExtractor {
    fn list(&self) -> Result<Vec<Entry>> {
        list_impl(&self.target, |file| convert(file))
    }

    fn target(&self) -> &PathBuf {
        &self.target
    }

    fn perform(&self, opts: &ExtractorOpts) -> Result<()> {
        let list = match list_impl(&self.target, |file| {
            (file.name().to_string(), file.uncompressed_size())
        }) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        let mut cabinet = open_cabinet(&self.target)?;
        for file in list {
            let file_name = file.0.clone();
            let dest_file = opts.base_dir(&self.target).join(&file_name);
            log::info!("extracting {} ({} bytes)", &file_name, file.1);
            create_dir_all(dest_file.parent().unwrap()).unwrap();
            let mut dest = match File::create(dest_file) {
                Ok(f) => f,
                Err(e) => return Err(ToteError::IO(e)),
            };
            let mut file_from = cabinet.read_file(&file_name).unwrap();
            match std::io::copy(&mut file_from, &mut dest) {
                Ok(_) => {}
                Err(e) => return Err(ToteError::IO(e)),
            }
        }
        Ok(())
    }

    fn format(&self) -> crate::format::Format {
        crate::format::Format::Cab
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
    let cabinet = open_cabinet(&archive_file)?;
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
    let mtime = f.datetime().map(|t| to_naive_datetime(t));
    Entry::new(name, None, Some(uncompressed_size as u64), None, mtime)
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
    use crate::format::Format;
    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("testdata/test.cab");
        let extractor = CabExtractor::new(file);
        match extractor.list() {
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
        let archive_file = PathBuf::from("testdata/test.cab");
        let e = CabExtractor::new(archive_file.clone());
        let dest = PathBuf::from("results/cab");
        let opts = ExtractorOpts::new_with_opts(Some(dest), true, true);
        match e.perform(&opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/cab/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/cab")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let e = CabExtractor::new(PathBuf::from("testdata/test.cab"));
        assert_eq!(e.format(), Format::Cab);
    }
}
