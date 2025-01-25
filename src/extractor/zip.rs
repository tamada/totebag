use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use zip::read::ZipFile;

use crate::extractor::{Entry, Extractor, ExtractorOpts};
use crate::format::Format;
use crate::Result;

pub(super) struct ZipExtractor {
    target: PathBuf,
}

impl ZipExtractor {
    pub(crate) fn new(file: PathBuf) -> Self {
        Self { target: file }
    }
}

impl Extractor for ZipExtractor {
    fn list(&self) -> Result<Vec<Entry>> {
        let zip_file = File::open(&self.target).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();

        let mut result = vec![];
        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();
            result.push(convert(file));
        }
        Ok(result)
    }

    fn perform(&self, opts: &ExtractorOpts) -> Result<()> {
        let zip_file = File::open(&self.target).unwrap();
        let mut zip = zip::ZipArchive::new(zip_file).unwrap();
        let dest_base = opts.base_dir(&self.target);
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            if file.is_file() {
                log::info!("extracting {} ({} bytes)", file.name(), file.size());
                let dest = dest_base.join(PathBuf::from(file.name().to_string()));
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

fn convert(zfile: ZipFile) -> Entry {
    let name = zfile.name().to_string();
    let compressed_size = zfile.compressed_size();
    let uncompresseed_size = zfile.size();
    let mode = zfile.unix_mode();
    let mtime = match zfile.last_modified() {
        Some(t) => convert_to_datetime(t),
        None => None,
    };
    Entry::new(
        name,
        Some(compressed_size),
        Some(uncompresseed_size),
        mode,
        mtime,
    )
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
    use std::path::PathBuf;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("testdata/test.zip");
        let extractor = ZipExtractor::new(file);
        match extractor.list() {
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
        let archive_file = PathBuf::from("testdata/test.zip");
        let e = ZipExtractor::new(archive_file.clone());
        let dest = PathBuf::from("results/zip");
        let opts = ExtractorOpts::new_with_opts(Some(dest), false, true);
        match e.perform(&opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/zip/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/zip")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn test_format() {
        let e = ZipExtractor::new(PathBuf::from("testdata/test.zip"));
        assert_eq!(e.format(), Format::Zip);
    }
}
