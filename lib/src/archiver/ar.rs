use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::archiver::{ArchiveEntry, Target, ToteArchiver};
use crate::{Error, Result};

/// AR format archiver implementation.
///
/// `ar` is a flat container: it stores files only, and has no representation for
/// directories. Directories encountered while walking the targets are therefore
/// skipped rather than written as zero-length members.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[Target],
        _config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut builder = ar::Builder::new(file);
        let mut errs = vec![];
        let mut entries = vec![];
        for t in targets {
            // ar has no directories; their files are archived on their own.
            if !t.path.is_file() {
                continue;
            }
            entries.push(ArchiveEntry::from(&t.path));
            if let Err(e) = process_file(&mut builder, &t.path, &t.dest_path) {
                errs.push(e);
            }
        }
        Error::error_or(entries, errs)
    }

    fn enable(&self) -> bool {
        true
    }
}

fn process_file(
    builder: &mut ar::Builder<&mut BufWriter<File>>,
    target: &Path,
    dest_path: &Path,
) -> Result<()> {
    let mut file = File::open(target).map_err(Error::IO)?;
    let name = dest_path.to_string_lossy().into_owned();
    builder
        .append_file(name.as_bytes(), &mut file)
        .map_err(Error::IO)
}

#[cfg(test)]
mod tests {
    use crate::archiver::test_support::targets;
    use std::path::PathBuf;

    fn run_test<F>(f: F)
    where
        F: FnOnce() -> PathBuf,
    {
        // setup(); // preprocessing process
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match result {
            Ok(path) => teardown(path),
            Err(err) => std::panic::resume_unwind(err),
        }
    }

    #[test]
    fn test_tar() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.ar")
                .overwrite(true)
                .build();
            let v = targets();
            let result = crate::archive(&v, &config);
            let path = PathBuf::from("results/test.ar");
            if let Err(e) = result {
                panic!("{:?}", e);
            }
            assert!(result.is_ok());
            assert!(path.exists());
            path
        });
    }

    fn teardown(path: PathBuf) {
        let _ = std::fs::remove_file(path);
    }
}
