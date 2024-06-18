use std::fs::{create_dir_all, File};
use std::io::copy;
use std::path::PathBuf;

use crate::cli::{Result, ToteError};
use crate::extractor::{Extractor, ExtractorOpts};

pub(super) struct LhaExtractor {}

impl Extractor for LhaExtractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>> {
        let mut result = Vec::<String>::new();
        let mut reader = match delharc::parse_file(&archive_file) {
            Err(e) => return Err(ToteError::IO(e)),
            Ok(h) => h,
        };
        loop {
            let header = reader.header();
            let name = header.parse_pathname();
            if !header.is_directory() {
                result.push(name.to_str().unwrap().to_string());
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

    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()> {
        let mut reader = match delharc::parse_file(&archive_file) {
            Err(e) => return Err(ToteError::IO(e)),
            Ok(h) => h,
        };
        loop {
            let header = reader.header();
            let name = header.parse_pathname();
            let dest = opts.destination(&archive_file).join(&name);
            if reader.is_decoder_supported() {
                opts.v.verbose(format!(
                    "extracting {} ({} bytes)",
                    &name.to_str().unwrap().to_string(),
                    header.original_size
                ));
                create_dir_all(dest.parent().unwrap()).unwrap();
                let mut dest = match File::create(dest) {
                    Ok(f) => f,
                    Err(e) => return Err(ToteError::IO(e)),
                };
                match copy(&mut reader, &mut dest) {
                    Ok(_) => {}
                    Err(e) => return Err(ToteError::IO(e)),
                }
                if let Err(e) = reader.crc_check() {
                    return Err(ToteError::Fatal(Box::new(e)));
                };
            } else if !header.is_directory() {
                opts.v.verbose(format!(
                    "{:?}: unsupported compression method ({:?})",
                    &name, header.compression
                ));
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
        Ok(())
    }

    fn format(&self) -> crate::format::Format {
        crate::format::Format::LHA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Format;
    use crate::verboser::create_verboser;

    #[test]
    fn test_list_archives() {
        let extractor = LhaExtractor {};
        let file = PathBuf::from("testdata/test.lzh");
        match extractor.list_archives(file) {
            Ok(r) => {
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
        let e = LhaExtractor {};
        let file = PathBuf::from("testdata/test.lzh");
        let opts = ExtractorOpts {
            dest: PathBuf::from("results/lha"),
            use_archive_name_dir: true,
            overwrite: true,
            v: create_verboser(false),
        };
        match e.perform(file, &opts) {
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
        let extractor = LhaExtractor {};
        assert_eq!(extractor.format(), Format::LHA);
    }
}
