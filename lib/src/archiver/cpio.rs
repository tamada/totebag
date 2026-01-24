use std::fs::File;
use std::path::PathBuf;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, ToteError};

/// CAB (Cabinet) format archiver implementation.
///
/// This archiver creates Microsoft Cabinet archive files.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(&self, file: File, targets: &[PathBuf], config: &crate::ArchiveConfig) -> Result<Vec<ArchiveEntry>> {
        let entries = super::collect_entries(targets, config);
        let mut builder = cpio::Builder::new(file);
        let mut errs = vec![];
        for path in entries.iter() {
            let path_in_archive = config.path_in_archive(path);
            if let Err(e) = builder.append_path(path, &path_in_archive) {
                errs.push(ToteError::Archiver(e.to_string()));
            };
        }
        match builder.finish() {
            Ok(_) => Ok(entries.into_iter().map(ArchiveEntry::from).collect()),
            Err(e) => {
                errs.push(ToteError::Archiver(e.to_string()));
                ToteError::error_or(vec![], errs)
            },
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn run_test<F>(f: F)
    where
        F: FnOnce(),
    {
        // setup(); // preprocessing
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        teardown(); // postprocessing

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_archive() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.cpio")
                .overwrite(false)
                .no_recursive(true)
                .build();
            let v = vec!["lib", "cli", "Cargo.toml"]
                .into_iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{e:?}")
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.cpio");
    }
}
