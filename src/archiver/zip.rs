#[cfg(target_os = "windows")]
use crate::archiver::os;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::archiver::os;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use zip::ZipWriter;

use crate::archiver::{ArchiveEntry, TargetPath, Targets, ToteArchiver};
use crate::{Result, ToteError};

pub(super) struct ZipArchiver {}

impl ZipArchiver {
    pub fn new() -> Self {
        Self {}
    }
    fn process_file(
        &self,
        zw: &mut ZipWriter<File>,
        target: PathBuf,
        tp: &TargetPath,
        level: u8,
    ) -> Result<()> {
        let opts = os::create_file_opts(&target, level as i64);
        let dest_path = tp.dest_path(&target);
        let name = dest_path.to_str().unwrap();
        if let Err(e) = zw.start_file(name, opts) {
            return Err(ToteError::Fatal(Box::new(e)));
        }
        let mut file = BufReader::new(File::open(target).unwrap());
        match std::io::copy(&mut file, zw) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToteError::IO(e)),
        }
    }
}

impl ToteArchiver for ZipArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let mut errs = vec![];
        let mut zw = zip::ZipWriter::new(file);
        let mut entries = vec![];
        for tp in tps.iter() {
            for t in tp.iter() {
                let path = t.into_path();
                entries.push(ArchiveEntry::from(&path));
                if path.is_file() {
                    if let Err(e) = self.process_file(&mut zw, path, tp, tps.level()) {
                        errs.push(e);
                    }
                }
            }
        }
        match zw.finish() {
            Ok(_) => Ok(entries),
            Err(e) => {
                errs.push(ToteError::Archiver(e.to_string()));
                Err(ToteError::Array(errs))
            }
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;
    use crate::archiver::Archiver;

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
                .archive_file(PathBuf::from("results/test.zip"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(true)
                .build();

            if let Err(e) = archiver.perform() {
                panic!("{:?}", e);
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.zip");
    }
}
