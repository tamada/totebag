use std::fs::File;
use std::path::PathBuf;

use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};

use crate::archiver::{Archiver, ArchiverOpts};
use crate::cli::{Result, ToteError};
use crate::format::Format;

pub(super) struct SevenZArchiver {}

impl Archiver for SevenZArchiver {
    fn perform(&self, opts: &ArchiverOpts) -> Result<()> {
        match opts.destination() {
            Err(e) => Err(e),
            Ok(file) => write_sevenz(file, opts.targets(), opts.recursive),
        }
    }

    fn format(&self) -> Format {
        Format::SevenZ
    }
}

fn process_file(szw: &mut SevenZWriter<File>, target: PathBuf) -> Result<()> {
    let name = target.to_str().unwrap();
    if let Err(e) = szw.push_archive_entry(
        SevenZArchiveEntry::from_path(&target, name.to_string()),
        Some(File::open(target).unwrap()),
    ) {
        return Err(ToteError::Archiver(e.to_string()));
    }
    Ok(())
}

fn process_dir(szw: &mut SevenZWriter<File>, target: PathBuf) -> Result<()> {
    for entry in target.read_dir().unwrap() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_dir() {
                process_dir(szw, e.path())?
            } else if p.is_file() {
                process_file(szw, e.path())?
            }
        }
    }
    Ok(())
}

fn write_sevenz_impl(
    mut szw: SevenZWriter<File>,
    targets: Vec<PathBuf>,
    recursive: bool,
) -> Result<()> {
    for target in targets {
        let path = target.as_path();
        if path.is_dir() && recursive {
            process_dir(&mut szw, path.to_path_buf())?
        } else {
            process_file(&mut szw, path.to_path_buf())?
        }
    }
    if let Err(e) = szw.finish() {
        return Err(ToteError::Archiver(e.to_string()));
    }
    Ok(())
}

fn write_sevenz(dest: File, targets: Vec<PathBuf>, recursive: bool) -> Result<()> {
    match SevenZWriter::new(dest) {
        Ok(write) => write_sevenz_impl(write, targets, recursive),
        Err(e) => Err(ToteError::Archiver(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
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
            let archiver = SevenZArchiver {};
            let inout = ArchiverOpts::create(
                PathBuf::from("results/test.7z"),
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")],
                true,
                true,
                false,
            );
            let result = archiver.perform(&inout);
            assert!(result.is_ok());
            assert_eq!(archiver.format(), Format::SevenZ);
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.7z");
    }
}
