use std::path::PathBuf;
use std::fs::{create_dir_all, File};

use cab::Cabinet;

use crate::{Result, ToteError};
use crate::extractor::ToteExtractor as Extractor;
use crate::extractor::ExtractorOpts;

pub(super) struct CabExtractor {}

fn list_impl<F, T>(archive_file: &PathBuf, mapper: F) -> Result<Vec<T>> 
        where F: Fn(&cab::FileEntry) -> T {
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
    fn list_archives(&self, archive_file: &PathBuf) -> Result<Vec<String>> {
        list_impl(&archive_file, |file| file.name().to_string())
    }

    fn perform(&self, archive_file: &PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let list = match list_impl(&archive_file, 
                |file| (file.name().to_string(), file.uncompressed_size())) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        let mut cabinet = open_cabinet(&archive_file)?;
        for file in list {
            let file_name = file.0.clone();
            let dest_file = opts.destination(&archive_file)?.join(&file_name);
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
    use crate::format::Format;
    use super::*;
    #[test]
    fn test_list_archives() {
        let extractor = CabExtractor{};
        let file = PathBuf::from("testdata/test.cab");
        match extractor.list_archives(&file) {
            Ok(r) => {
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(2), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let e = CabExtractor{};
        let file = PathBuf::from("testdata/test.cab");
        let opts = ExtractorOpts {
            dest: PathBuf::from("results/cab"),
            use_archive_name_dir: true,
            overwrite: true,
        };
        match e.perform(&file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/cab/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/cab")).unwrap();
            },
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let extractor = CabExtractor {};
        assert_eq!(extractor.format(), Format::Cab);
    }
}