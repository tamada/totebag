use std::fs::File;
use std::path::PathBuf;

use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};

use crate::archiver::TargetPath;
use crate::archiver::ToteArchiver;
use crate::format::Format;
use crate::{Result, ToteError};

pub(super) struct SevenZArchiver {}

impl ToteArchiver for SevenZArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>) -> Result<()> {
        let mut w = match SevenZWriter::new(file) {
            Ok(writer) => writer,
            Err(e) => return Err(ToteError::Archiver(e.to_string())),
        };
        let mut errs = vec![];
        for tp in tps {
            for entry in tp.walker() {
                if let Ok(t) = entry {
                    let path = t.into_path();
                    if path.is_file() {
                        if let Err(e) = process_file(&mut w, &path, &tp.dest_path(&path)) {
                            errs.push(e);
                        }
                    }
                }
            }
        }
        if let Err(e) = w.finish() {
            errs.push(ToteError::Archiver(e.to_string()));
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(ToteError::Array(errs))
        }
    }

    fn format(&self) -> Format {
        Format::SevenZ
    }

    fn enable(&self) -> bool {
        true
    }
}

fn process_file(szw: &mut SevenZWriter<File>, target: &PathBuf, dest_path: &PathBuf) -> Result<()> {
    let name = &dest_path.to_str().unwrap();
    if let Err(e) = szw.push_archive_entry(
        SevenZArchiveEntry::from_path(&dest_path, name.to_string()),
        Some(File::open(target).unwrap()),
    ) {
        return Err(ToteError::Archiver(e.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::archiver::Archiver;

    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_format() {
        let archiver = SevenZArchiver {};
        assert_eq!(archiver.format(), Format::SevenZ);
    }

    fn run_test<F>(f: F)
    where
        F: FnOnce(),
    {
        // setup(); // 予めやりたい処理
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        teardown(); // 後片付け処理

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_zip() {
        run_test(|| {
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.7z"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();
            let result = archiver.perform();
            assert!(result.is_ok());
            assert_eq!(Format::SevenZ, archiver.format());
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.7z");
    }
}
