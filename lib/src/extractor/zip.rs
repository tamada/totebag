use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use zip::read::ZipFile;

use crate::extractor::{Entry, PathUtils, ToteExtractor};
use crate::Result;

pub(super) struct ZipExtractor {}

impl ToteExtractor for ZipExtractor {
    fn list(&self, archive_file: PathBuf) -> Result<Vec<Entry>> {
        let zip_file = File::open(archive_file).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();

        let mut result = vec![];
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();
            result.push(convert(file));
        }
        Ok(result)
    }

    fn perform(&self, archive_file: PathBuf, opts: PathUtils) -> Result<()> {
        let zip_file = File::open(archive_file).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            if file.is_file() {
                log::info!("extracting {} ({} bytes)", file.name(), file.size());
                let dest = opts
                    .destination(PathBuf::from(file.name().to_string()))
                    .unwrap();
                create_dir_all(dest.parent().unwrap()).unwrap();
                let mut out = File::create(dest).unwrap();
                copy(&mut file, &mut out).unwrap();
            }
        }
        Ok(())
    }
}

fn convert(zfile: ZipFile) -> Entry {
    let name = zfile.name().to_string();
    let compressed_size = zfile.compressed_size();
    let uncompresseed_size = zfile.size();
    let mode = zfile.unix_mode();
    let mtime = match zfile.last_modified() {
        Some(t) => convert_to_datetime(t),
        None => None,
    };
    Entry::builder()
        .name(name)
        .compressed_size(compressed_size)
        .original_size(uncompresseed_size)
        .unix_mode(mode.unwrap())
        .date(mtime.unwrap())
        .build()
}

fn convert_to_datetime(t: zip::DateTime) -> Option<NaiveDateTime> {
    use chrono::NaiveDate;

    let year = t.year() as i32;
    let month = t.month() as u32;
    let day = t.day() as u32;
    let hour = t.hour() as u32;
    let minute = t.minute() as u32;
    let second = t.second() as u32;
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, minute, second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::Extractor;
    use std::path::PathBuf;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("../testdata/test.zip");
        let extractor = ZipExtractor {};
        match extractor.list(file) {
            Ok(r) => {
                assert_eq!(r.len(), 19);
                assert_eq!(
                    r.get(0).map(|t| &t.name),
                    Some("Cargo.toml".to_string()).as_ref()
                );
                assert_eq!(
                    r.get(1).map(|t| &t.name),
                    Some("build.rs".to_string()).as_ref()
                );
                assert_eq!(
                    r.get(2).map(|t| &t.name),
                    Some("LICENSE".to_string()).as_ref()
                );
                assert_eq!(
                    r.get(3).map(|t| &t.name),
                    Some("README.md".to_string()).as_ref()
                );
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.zip");
        let opts = Extractor::builder()
            .archive_file(archive_file)
            .destination("results/zip")
            .build();
        match opts.perform() {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/zip/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/zip")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
