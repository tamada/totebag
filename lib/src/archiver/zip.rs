use crate::archiver::os;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use zip::ZipWriter;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Error, Result};

/// ZIP format archiver implementation.
///
/// This archiver creates ZIP archive files.
pub(super) struct Archiver {}

impl Archiver {
    fn process_file(
        &self,
        zw: &mut ZipWriter<File>,
        target: &Path,
        dest_path: PathBuf,
        level: u8,
    ) -> Result<()> {
        let opts = os::create_file_opts(target, level as i64)?;
        let name = dest_path.to_string_lossy().into_owned();
        if let Err(e) = zw.start_file(name, opts) {
            Err(Error::Fatal(Box::new(e)))
        } else {
            let mut file = BufReader::new(File::open(target).map_err(Error::IO)?);
            match std::io::copy(&mut file, zw) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::IO(e)),
            }
        }
    }
}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut errs = vec![];
        let mut zw = ZipWriter::new(file);
        let mut entries = vec![];
        for tp in targets.iter() {
            for entry in config.iter(tp) {
                let path = entry.path().to_path_buf();
                entries.push(ArchiveEntry::from(&path));
                if path.is_file()
                    && let Err(e) = self.process_file(
                        &mut zw,
                        &path,
                        config.path_in_archive(&path),
                        config.level,
                    )
                {
                    errs.push(e);
                }
            }
        }
        match zw.finish() {
            Ok(_) => Ok(entries),
            Err(e) => {
                errs.push(Error::Archiver(e.to_string()));
                Err(Error::Array(errs))
            }
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::test_support::targets;
    fn run_test<F>(f: F)
    where
        F: FnOnce(),
    {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        teardown();

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_zip() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.zip")
                .overwrite(true)
                .build();
            let v = targets();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{e:?}")
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.zip");
    }
}
