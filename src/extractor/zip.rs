use std::fs::{File, create_dir_all};
use std::io::copy;
use std::path::PathBuf;

use crate::{cli::Result, format::Format};
use crate::extractor::{ExtractorOpts, Extractor};


pub(super) struct ZipExtractor {
}

impl Extractor for  ZipExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let zip_file = File::open(archive_file).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();

        let mut result = Vec::<String>::new();
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();
            result.push(file.name().to_string());
            // std::io::copy(&mut file, &mut std::io::stdout()).unwrap();
        }
        Ok(result)
    }

    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let zip_file = File::open(&archive_file).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            if file.is_file() {
                opts.v.verbose(format!("extracting {} ({} bytes)", file.name(), file.size()));
                let dest = opts.destination(&archive_file).join(PathBuf::from(file.name().to_string()));
                create_dir_all(dest.parent().unwrap()).unwrap();
                let mut out = File::create(dest).unwrap();
                copy(&mut file, &mut out).unwrap();
            }
        }
        Ok(())
    }

    fn format(&self) -> Format {
        Format::Zip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_list_archives() {
        let extractor = ZipExtractor{};
        let file = PathBuf::from("testdata/test.zip");
        match extractor.list_archives(file) {
            Ok(r) => {
                assert_eq!(r.len(), 19);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            },
            Err(_) => assert!(false),
        }
    }
}