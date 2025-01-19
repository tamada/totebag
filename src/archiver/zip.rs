#[cfg(target_os = "windows")]
use crate::archiver::os::windows::*;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::archiver::os::linux::*;

use std::fs::File;
use std::path::PathBuf;
use std::io::BufReader;
use zip::ZipWriter;

use crate::archiver::{ToteArchiver, Format, ArchiverOpts};
use crate::{ToteError, Result};

use super::TargetPath;

pub(super) struct ZipArchiver {
}

impl ZipArchiver {
    pub fn new() -> Self {
        Self { }
    }
    fn process_file(&self, zw: &mut ZipWriter<File>, target: PathBuf, tp: &TargetPath) -> Result<()> {
        let opts = create_file_opts(&target);
        let dest_path = tp.dest_path(&target);
        let name = dest_path.to_str().unwrap();
        if let Err(e) = zw.start_file(name, opts) {
            return Err(ToteError::Fatal(Box::new(e)));
        }
        let mut file = BufReader::new(File::open(target).unwrap());
        match std::io::copy(&mut file, zw) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToteError::IO(e))
        }
    }
}

impl ToteArchiver for ZipArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        let mut errs = vec![];
        let mut zw = zip::ZipWriter::new(file);
        for tp in tps {
            for entry in tp.walker() {
                if let Ok(t) = entry {
                    let path = t.into_path();
                    if path.is_file() {
                        if let Err(e) = self.process_file(&mut zw, path, &tp) {
                            errs.push(e);
                        }
                    }
                }
            }
        }
        match zw.finish() {
            Ok(_) => Ok(()),
            Err(e) => {
                errs.push(ToteError::Archiver(e.to_string()));
                Err(ToteError::Array(errs))
            }
        }
    }

    fn enable(&self) -> bool {
        true
    }
    
    fn format(&self) -> Format {
        Format::Zip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archiver::{Archiver, ArchiverOpts};

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
            let opts = ArchiverOpts::create(None, true, true, vec![]);
            let archiver = Archiver::new(PathBuf::from("results/test.zip"), vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")], opts).unwrap();
            let result = archiver.perform();
            assert!(result.is_ok());
            assert_eq!(archiver.format(), Format::Zip);
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.zip");
    }
}