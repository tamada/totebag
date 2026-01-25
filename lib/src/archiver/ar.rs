use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, Error};

/// TAR format archiver implementation.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(&self, file: File, targets: &[PathBuf], config: &crate::ArchiveConfig) -> Result<Vec<ArchiveEntry>> {
        let mut builder = ar::Builder::new(file);
        let mut errs = vec![];
        let mut entries = vec![];
        for tp in targets {
            for entry in config.iter(tp) {
                let path = entry.into_path();
                entries.push(ArchiveEntry::from(&path));
                let dest_dir = config.path_in_archive(&path);
                if path.is_file() {
                    if let Err(e) = process_file(&mut builder, &path, &dest_dir) {
                        errs.push(e);
                    }
                } else if path.is_dir() {
                    if let Err(e) = append_dir(&mut builder, &dest_dir, &path) {
                        errs.push(e);
                    }
                }
            }
        }
        Error::error_or(entries, errs)
    }

    fn enable(&self) -> bool {
        true
    }
}

fn append_dir<W: Write>(builder: &mut ar::Builder<W>, dest_path: &Path, src_path: &Path) -> Result<()> {
    let identifier = dest_path.to_str().unwrap().to_string();
    let metadata = std::fs::metadata(src_path).map_err(Error::IO)?;
    let header = ar::Header::from_metadata(identifier.into_bytes(), &metadata);
    builder.append(&header, &mut std::io::empty()).map_err(Error::IO)
}

fn process_file<W: Write>(builder: &mut ar::Builder<W>, target: &Path, dest_path: &Path) -> Result<()> {
    match std::fs::File::open(target) {
        Err(e) => Err(Error::IO(e)),
        Ok(mut file) => {
            let name = dest_path.to_str().unwrap();
            builder.append_file(name.as_bytes(), &mut file)
                .map_err(Error::IO)
        }
    }
}

#[cfg(test)]
mod tests {
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
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<_>>();
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
