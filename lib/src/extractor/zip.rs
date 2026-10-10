use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use zip::read::ZipFile;

use crate::extractor::{Entries, Entry, ToteExtractor};
use crate::{Error, Result};

/// The unix mode recorded for entries whose archive does not carry one
/// (a zip written on Windows, for example).
const DEFAULT_UNIX_MODE: u32 = 0o644;

/// ZIP format extractor implementation.
///
/// This extractor handles ZIP archive files.
pub(super) struct Extractor {}

fn open(archive_file: &PathBuf) -> Result<zip::ZipArchive<File>> {
    let zip_file = File::open(archive_file).map_err(Error::IO)?;
    zip::ZipArchive::new(zip_file).map_err(|e| Error::Extractor(e.to_string()))
}

impl ToteExtractor for Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        let mut zip = open(&archive_file)?;
        let mut result = vec![];
        for i in 0..zip.len() {
            let file = zip
                .by_index(i)
                .map_err(|e| Error::Extractor(e.to_string()))?;
            result.push(convert(file));
        }
        Ok(Entries::new(archive_file, result))
    }

    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        let mut errs = vec![];
        let mut zip = open(&archive_file)?;
        for i in 0..zip.len() {
            let file = zip.by_index(i).map_err(|e| Error::Extractor(e.to_string()));
            match file {
                Ok(mut file) => {
                    if let Err(e) = perform_impl(&mut file, &base) {
                        errs.push(e);
                    }
                }
                Err(e) => errs.push(e),
            }
        }
        Error::error_or((), errs)
    }
}

fn perform_impl<R: std::io::Read>(file: &mut ZipFile<R>, base: &Path) -> Result<u64> {
    if !file.is_file() {
        return Ok(0);
    }
    log::info!("extracting {} ({} bytes)", file.name(), file.size());
    let dest = super::safe_join(base, Path::new(file.name()))?;
    super::create_parent_dir_all(&dest)?;
    let mut out = File::create(dest).map_err(Error::IO)?;
    copy(file, &mut out).map_err(Error::IO)
}

fn convert<R: std::io::Read>(zfile: ZipFile<R>) -> Entry {
    Entry::builder()
        .name(zfile.name().to_string())
        .compressed_size(zfile.compressed_size())
        .original_size(zfile.size())
        .unix_mode(zfile.unix_mode().unwrap_or(DEFAULT_UNIX_MODE))
        .date(zfile.last_modified().and_then(convert_to_datetime))
        .build()
}

fn convert_to_datetime(t: zip::DateTime) -> Option<NaiveDateTime> {
    use chrono::NaiveDate;

    NaiveDate::from_ymd_opt(
        i32::from(t.year()),
        u32::from(t.month()),
        u32::from(t.day()),
    )?
    .and_hms_opt(
        u32::from(t.hour()),
        u32::from(t.minute()),
        u32::from(t.second()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("../testdata/test.zip");
        let extractor = Extractor {};
        match extractor.list(file) {
            Ok(r) => {
                assert_eq!(r.len(), 19);
                let mut i = r.iter();
                assert_eq!(
                    i.next().map(|t| &t.name),
                    Some("Cargo.toml".to_string()).as_ref()
                );
                assert_eq!(
                    i.next().map(|t| &t.name),
                    Some("build.rs".to_string()).as_ref()
                );
                assert_eq!(
                    i.next().map(|t| &t.name),
                    Some("LICENSE".to_string()).as_ref()
                );
                assert_eq!(
                    i.next().map(|t| &t.name),
                    Some("README.md".to_string()).as_ref()
                );
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.zip");
        let opts = crate::ExtractConfig::builder().dest("results/zip").build();
        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(PathBuf::from("results/zip/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/zip")).unwrap();
            }
            Err(e) => panic!("unexpected error: {e:?}"),
        };
    }

    #[test]
    fn test_extract_rejects_unsafe_paths() {
        let dir = PathBuf::from("results/zip_unsafe");
        let _ = std::fs::remove_dir_all(&dir);
        let opts = crate::ExtractConfig::builder()
            .dest(dir.join("out"))
            .build();
        let r = crate::extract("../testdata/unsafe/unsafe.zip", &opts);

        // The safe entries on both sides of the unsafe ones are extracted...
        assert!(dir.join("out/ok_before.txt").exists());
        assert!(dir.join("out/ok_after.txt").exists());
        // ...nothing is written next to `out`...
        assert!(!dir.join("evil.txt").exists());
        // ...and both unsafe entries are reported.
        match r {
            Err(Error::Array(errs)) => {
                assert_eq!(errs.len(), 2);
                assert!(errs.iter().all(|e| matches!(e, Error::UnsafePath(..))));
            }
            other => panic!("unexpected result: {other:?}"),
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
