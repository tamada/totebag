use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use cab::Cabinet;

use crate::extractor::ExtractorOpts;
use crate::extractor::ToteExtractor as Extractor;
use crate::{Result, ToteError};

pub(super) struct CabExtractor {}

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

impl Extractor for CabExtractor {
    fn list(&self, archive_file: &PathBuf) -> Result<Vec<String>> {
        list_impl(&archive_file, |file| file.name().to_string())
    }

    fn perform(&self, archive_file: &PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let list = match list_impl(&archive_file, |file| {
            (file.name().to_string(), file.uncompressed_size())
        }) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        let mut cabinet = open_cabinet(&archive_file)?;
        for file in list {
            let file_name = file.0.clone();
            let dest_file = opts.base_dir(archive_file).join(&file_name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Format;
    #[test]
    fn test_list_archives() {
        let extractor = CabExtractor {};
        let file = PathBuf::from("testdata/test.cab");
        match extractor.list(&file) {
            Ok(r) => {
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
        let e = CabExtractor {};
        let archive_file = PathBuf::from("testdata/test.cab");
        let dest = PathBuf::from("results/cab");
        let opts = ExtractorOpts::new_with_opts(Some(dest), true, true);
        match e.perform(&archive_file, &opts) {
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
        let extractor = CabExtractor {};
        assert_eq!(extractor.format(), Format::Cab);
    }
}
