#[cfg(target_os = "windows")]
use os::windows::*;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use os::linux::*;

use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, Write, Seek};
use zip::ZipWriter;

use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::archiver::os;
use crate::cli::{ToteError, Result};

pub(super) struct ZipArchiver {
}

impl Archiver for  ZipArchiver {
    fn perform(&self, inout: &ArchiverOpts) -> Result<()> {
        match inout.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                write_to_zip(file, inout.targets(), inout.recursive, inout.base_dir.clone())
            }
        }
    }
    
    fn format(&self) -> Format {
        Format::Zip
    }
}

fn process_dir<W:Write+Seek> (zw: &mut ZipWriter<W>, target: PathBuf, base_dir: &PathBuf) -> Result<()> {
    for entry in target.read_dir().unwrap() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_dir() {
                process_dir(zw, e.path(), &base_dir)?
            } else if p.is_file() {
                process_file(zw, e.path(), &base_dir)?
            }
        }
    }
    Ok(())
}

fn process_file<W:Write+Seek> (zw: &mut ZipWriter<W>, target: PathBuf, base_dir: &PathBuf) -> Result<()> {
    let target_path = match target.strip_prefix(base_dir) {
        Ok(p) => p.to_path_buf(),
        Err(_) => target.clone(),
    };
    let name = target_path.to_str().unwrap();
    let opts = create(&target);
    if let Err(e) = zw.start_file(name, opts) {
        return Err(ToteError::Archiver(e.to_string()));
    }
    let mut file = BufReader::new(File::open(target).unwrap());
    if let Err(e) = std::io::copy(&mut file, zw) {
        return Err(ToteError::IO(e))
    }
    Ok(())
}

fn write_to_zip(dest: File, targets: Vec<PathBuf>, recursive: bool, base_dir: PathBuf) -> Result<()> {
    let mut zw = zip::ZipWriter::new(dest);
    for target in targets {
        let path = target.as_path();
        if path.is_dir() && recursive {
            process_dir(&mut zw, path.to_path_buf(), &base_dir)?
        } else {
            process_file(&mut zw, path.to_path_buf(), &base_dir)?
        }
    }
    if let Err(e) = zw.finish() {
        return Err(ToteError::Archiver(e.to_string()));
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
            let inout = ArchiverOpts::create(PathBuf::from("results/test.zip"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], PathBuf::from("."), true, true, false);
            let result = archiver.perform(&inout);
            assert!(result.is_ok());
            assert_eq!(archiver.format(), Format::Zip);
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.zip");
    }
}