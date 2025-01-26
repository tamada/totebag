use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::PathBuf;

use chrono::DateTime;
use delharc::{LhaDecodeReader, LhaHeader};

use crate::extractor::ToteExtractor;
use crate::extractor::{Entry, PathUtils};
use crate::{Result, ToteError};

pub(super) struct LhaExtractor {}

impl ToteExtractor for LhaExtractor {
    fn list(&self, archive_file: &PathBuf) -> Result<Vec<Entry>> {
        let mut result = vec![];
        let mut reader = match delharc::parse_file(&archive_file) {
            Err(e) => return Err(ToteError::IO(e)),
            Ok(h) => h,
        };
        loop {
            let header = reader.header();
            if !header.is_directory() {
                result.push(convert(header));
            }
            match reader.next_file() {
                Ok(r) => {
                    if !r {
                        break;
                    }
                }
                Err(e) => return Err(ToteError::Fatal(Box::new(e))),
            }
        }
        Ok(result)
    }

    fn perform(&self, archive_file: &PathBuf, opts: PathUtils) -> Result<()> {
        let mut reader = match delharc::parse_file(&archive_file) {
            Err(e) => return Err(ToteError::IO(e)),
            Ok(h) => h,
        };
        let mut errs = vec![];
        loop {
            if let Err(e) = write_data_impl(&mut reader, &opts) {
                errs.push(e);
            }
            match reader.next_file() {
                Ok(r) => {
                    if !r {
                        break;
                    }
                }
                Err(e) => return Err(ToteError::Fatal(Box::new(e))),
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(ToteError::Array(errs))
        }
    }

    #[cfg(test)]
    fn format(&self) -> crate::format::Format {
        crate::format::Format::LHA
    }
}

fn write_data_impl(reader: &mut LhaDecodeReader<File>, opts: &PathUtils) -> Result<()> {
    let header = reader.header();
    let name = header.parse_pathname();
    let dest = match opts.destination(name.clone()) {
        Ok(dest) => dest,
        Err(e) => return Err(e),
    };
    if reader.is_decoder_supported() {
        log::info!("extracting {:?} ({} bytes)", &name, header.original_size);
        create_dir_all(dest.parent().unwrap()).unwrap();
        let mut dest = match File::create(dest) {
            Ok(f) => f,
            Err(e) => return Err(ToteError::IO(e)),
        };
        match copy(reader, &mut dest) {
            Ok(_) => {}
            Err(e) => return Err(ToteError::IO(e)),
        }
        if let Err(e) = reader.crc_check() {
            return Err(ToteError::Fatal(Box::new(e)));
        };
    } else if !header.is_directory() {
        log::info!(
            "{:?}: unsupported compression method ({:?})",
            &name,
            header.compression
        );
    }
    Ok(())
}

fn convert(h: &LhaHeader) -> Entry {
    let name = h.parse_pathname().to_str().unwrap().to_string();
    let compressed_size = h.compressed_size;
    let original_size = h.original_size;
    let mtime = h.last_modified as i64;
    let dt = DateTime::from_timestamp(mtime, 0);
    Entry::new(
        name,
        Some(compressed_size),
        Some(original_size),
        None,
        dt.map(|dt| dt.naive_local()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::Extractor;
    use crate::format::Format;

    #[test]
    fn test_list_archives() {
        let file = PathBuf::from("testdata/test.lzh");
        let extractor = LhaExtractor {};
        match extractor.list(&file) {
            Ok(r) => {
                let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                assert_eq!(r.len(), 23);
                assert_eq!(r.get(0), Some("Cargo.toml".to_string()).as_ref());
                assert_eq!(r.get(1), Some("LICENSE".to_string()).as_ref());
                assert_eq!(r.get(2), Some("README.md".to_string()).as_ref());
                assert_eq!(r.get(3), Some("build.rs".to_string()).as_ref());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_extract_archive() {
        let archive_file = PathBuf::from("testdata/test.lzh");
        let opts = Extractor::builder()
            .archive_file(archive_file)
            .destination("results/lha")
            .use_archive_name_dir(true)
            .overwrite(true)
            .build();
        match opts.perform() {
            Ok(_) => {
                assert!(true);
                assert!(PathBuf::from("results/lha/test/Cargo.toml").exists());
                std::fs::remove_dir_all(PathBuf::from("results/lha")).unwrap();
            }
            Err(e) => {
                eprintln!("{:?}", e);
                assert!(false);
            }
        };
    }

    #[test]
    fn test_format() {
        let e = LhaExtractor {};
        assert_eq!(e.format(), Format::LHA);
    }
}
