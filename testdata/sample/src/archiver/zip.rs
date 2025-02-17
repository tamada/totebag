use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Write, Seek};
use time::OffsetDateTime;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, DateTime};

use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::cli::{ToatError, Result};

pub(super) struct ZipArchiver {
}

impl Archiver for  ZipArchiver {
    fn perform(&self, inout: ArchiverOpts) -> Result<()> {
        match inout.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                write_to_zip(file, inout.targets(), inout.recursive)
            }
        }
    }
    
    fn format(&self) -> Format {
        Format::Zip
    }
}

fn process_dir<W:Write+Seek> (zw: &mut ZipWriter<W>, target: PathBuf) -> Result<()> {
    for entry in target.read_dir().unwrap() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_dir() {
                process_dir(zw, e.path())?
            } else if p.is_file() {
                process_file(zw, e.path())?
            }
        }
    }
    Ok(())
}

fn process_file<W:Write+Seek> (zw: &mut ZipWriter<W>, target: PathBuf) -> Result<()> {
    let name = target.to_str().unwrap();
    let metadata = std::fs::metadata(&target).unwrap();
    let mod_time = DateTime::try_from(
        OffsetDateTime::from(metadata.modified().unwrap()));
    let opts = SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(metadata.permissions().mode());
    if let Err(e) = zw.start_file(name, opts) {
        return Err(ToatError::ArchiverError(e.to_string()));
    }
    let mut file = BufReader::new(File::open(target).unwrap());
    if let Err(e) = std::io::copy(&mut file, zw) {
        return Err(ToatError::IOError(e))
    }
    Ok(())
}

fn write_to_zip(dest: File, targets: Vec<PathBuf>, recursive: bool) -> Result<()> {
    let mut zw = zip::ZipWriter::new(dest);
    for target in targets {
        let path = target.as_path();
        if path.is_dir() && recursive {
            process_dir(&mut zw, path.to_path_buf())?
        } else {
            process_file(&mut zw, path.to_path_buf())?
        }
    }
    if let Err(e) = zw.finish() {
        return Err(ToatError::ArchiverError(e.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let archiver = ZipArchiver{};
            let inout = ArchiverOpts::create(PathBuf::from("test.zip"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], true, true, false);
            let result = archiver.perform(inout);
            assert!(result.is_ok());
            assert_eq!(archiver.format(), Format::Zip);
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("test.zip");
    }
}