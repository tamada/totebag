use std::fs::create_dir_all;
use std::io::Read;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::{Result, Error};
use ar::Archive;

use crate::extractor::{Entry as ToteEntry, Entries, ToteExtractor};

/// AR ormat extractor implementation.
pub(super) struct Extractor {}

impl ToteExtractor for Extractor {
    fn list(&self, archive_file: PathBuf) -> Result<Entries> {
        File::open(&archive_file)
            .map_err(Error::IO)
            .map(Archive::new)
            .and_then(|archive| list_ar(archive, archive_file))
    }
    fn perform(&self, archive_file: PathBuf, base: PathBuf) -> Result<()> {
        File::open(&archive_file)
            .map_err(Error::IO)
            .map(Archive::new)
            .and_then(|archive| extract_ar(archive, base))
    }
}

fn extract_ar<R: Read>(mut archive: ar::Archive<R>, base: PathBuf) -> Result<()> {
    let mut errs = vec![];
    while let Some(entry) = archive.next_entry() {
        let mut entry = match entry {
            Ok(e) => e,
            Err(e) => { errs.push(Error::IO(e)); continue; }
        };
        let header = entry.header();
        let path = match str::from_utf8(header.identifier()) {
            Ok(p) => PathBuf::from(p),
            Err(e) => {
                errs.push(Error::Archiver(e.to_string()));
                continue;
            }
        };
        if is_filename_mac_finder_file(&path) {
            continue;
        }
        let size = header.size();
        log::info!("extracting {path:?} ({size} bytes)");

        let dest = base.join(&path);
        if is_file(header.mode()) {
            if let Err(e) = write_to(&mut entry, &dest, &mut errs) {
                errs.push(e);
            }
        }
    }
    Ok(())
}

fn write_to<R: Read>(entry: &mut ar::Entry<R>, dest: &Path, errs: &mut Vec<Error>) -> Result<()> {
    create_dir_all(dest.parent().unwrap()).unwrap();
    let mut dest_file = File::create(dest).map_err(Error::IO)?;
    if let Err(e) = std::io::copy(entry, &mut dest_file) {
        errs.push(Error::IO(e));
    }
    Ok(())
}

fn is_file(mode: u32) -> bool {
    mode & 0o170000 == 0o100000
}

fn is_filename_mac_finder_file(path: &Path) -> bool {
    let filename = path.file_name().unwrap().to_str().unwrap();
    filename == ".DS_Store" || filename.starts_with("._")
}

fn list_ar<R: Read>(mut archive: ar::Archive<R>, path: PathBuf) -> Result<Entries> {
    let mut result = vec![];
    let mut errs = vec![];
    while let Some(entry) = archive.next_entry() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => { errs.push(Error::IO(e)); continue; }
        };
        result.push(convert_to_entry(entry.header()));
    }
    Error::error_or_else(|| Entries::new(path, result), errs)
}

fn convert_to_entry(e: &ar::Header) -> ToteEntry {
    let path = str::from_utf8(e.identifier()).unwrap();
    let size = e.size();
    let mode = e.mode();
    let mtime = e.mtime();
    let datetime = chrono::DateTime::from_timestamp_millis(mtime as i64);
    ToteEntry::builder()
        .name(path)
        .original_size(size)
        .unix_mode(mode)
        .date(datetime.map(|dt| dt.naive_local()))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tar_file() {
        let file = PathBuf::from("../testdata/test.ar");
        let extractor = Extractor {};
        match extractor.list(file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 16);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("build.rs".to_string()).as_ref());
                assert_eq!(r.get(2), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(3), Some("README.md".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("../testdata/test.ar");
        let dest = PathBuf::from("results/ar");
        let opts = crate::ExtractConfig::builder()
            .dest(dest)
            .use_archive_name_dir(true)
            .build();

        match crate::extract(archive_file, &opts) {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/ar/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/ar")).unwrap();
            }
            Err(_) => assert!(false),
        };
    }
}
