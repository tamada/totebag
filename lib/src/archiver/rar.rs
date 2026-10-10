use std::fs::File;
use std::path::{Path, PathBuf};

use rars::{ArchiveVersion, Builder, EntrySource, WriterResources};

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Error, Result};

/// RAR format archiver implementation.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        mut file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let level = level_to_rar(config.level);
        let mut builder = Builder::new(ArchiveVersion::Rar50).compression_level(Some(level));
        let mut errs = vec![];
        let mut entries = vec![];
        for tp in targets {
            for entry in config.iter(tp) {
                let path = entry.into_path();
                let dest_path = config.path_in_archive(&path);
                if dest_path.as_os_str().is_empty() {
                    continue;
                }
                entries.push(ArchiveEntry::from(&path));
                if path.is_file() {
                    if let Err(e) = process_file(&mut builder, &path, &dest_path) {
                        errs.push(e);
                    }
                } else if path.is_dir()
                    && let Err(e) = process_dir(&mut builder, &path, &dest_path)
                {
                    errs.push(e);
                }
            }
        }
        if let Err(e) = builder.write_to(&mut file, &WriterResources::default(), None) {
            errs.push(Error::Archiver(e.to_string()))
        }
        Error::error_or(entries, errs)
    }

    fn enable(&self) -> bool {
        true
    }
}

/// The RAR5 member name for `dest`: its components joined with `/`, which is
/// RAR's separator on every platform, then mapped with `encode_rar50`.
fn archive_name(dest: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for (i, component) in dest.components().enumerate() {
        if i > 0 {
            bytes.push(b'/');
        }
        let part = rars::filename::native_bytes(component.as_os_str())
            .map_err(|e| Error::Archiver(e.to_string()))?;
        bytes.extend_from_slice(part);
    }
    Ok(rars::filename::encode_rar50(&bytes).into_owned())
}

fn process_dir(builder: &mut rars::Builder, path: &Path, dest_path: &Path) -> Result<()> {
    let name = archive_name(dest_path)?;
    let (mtime, mode) = mtime_and_mode(path);
    builder
        .add_directory(name, mtime, mode)
        .map_err(|e| Error::Archiver(e.to_string()))
}

fn process_file(builder: &mut rars::Builder, path: &Path, dest_path: &Path) -> Result<()> {
    let name = archive_name(dest_path)?;
    let (mtime, mode) = mtime_and_mode(path);
    let source = EntrySource::from_path(path);
    builder
        .add_source(name, source, mtime, mode)
        .map_err(|e| Error::Archiver(e.to_string()))
}

fn mtime_and_mode(path: &Path) -> (Option<u32>, Option<u32>) {
    let Ok(metadata) = path.metadata() else {
        return (None, None);
    };
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|d| u32::try_from(d.as_secs()).ok());
    (mtime, unix_mode(&metadata))
}

#[cfg(unix)]
fn unix_mode(metadata: &std::fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.mode())
}

#[cfg(not(unix))]
fn unix_mode(_: &std::fs::Metadata) -> Option<u32> {
    None
}

/// Maps the totebag level (0-9) onto RAR's 0-5 (0 stores, 3 is RAR's default).
/// Values above 9 are treated as 9.
fn level_to_rar(level: u8) -> u8 {
    (u16::from(level.min(9)) * 5).div_ceil(9) as u8
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
    fn test_rar_level() {
        assert_eq!(super::level_to_rar(0), 0);
        assert_eq!(super::level_to_rar(1), 1);
        assert_eq!(super::level_to_rar(2), 2);
        assert_eq!(super::level_to_rar(3), 2);
        assert_eq!(super::level_to_rar(4), 3);
        assert_eq!(super::level_to_rar(5), 3);
        assert_eq!(super::level_to_rar(6), 4);
        assert_eq!(super::level_to_rar(7), 4);
        assert_eq!(super::level_to_rar(8), 5);
        assert_eq!(super::level_to_rar(9), 5);
        assert_eq!(super::level_to_rar(200), 5);
    }

    #[test]
    fn test_rar() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.rar")
                .overwrite(true)
                .build();
            let v = targets();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{e:?}")
            }
            let file = PathBuf::from("results/test.rar");
            let extractor = crate::extractor::create(&file).expect("extractor creation failed");
            match extractor.list(file) {
                Ok(r) => {
                    let r = r.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
                    assert_eq!(r.len(), 20);
                    assert_eq!(r.first(), Some("testdata/sample".to_string()).as_ref());
                    assert_eq!(
                        r.get(1),
                        Some("testdata/sample/Cargo.toml".to_string()).as_ref()
                    );
                    assert_eq!(
                        r.get(2),
                        Some("testdata/sample/LICENSE".to_string()).as_ref()
                    );
                    assert_eq!(
                        r.get(3),
                        Some("testdata/sample/build.rs".to_string()).as_ref()
                    );
                }
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.rar");
    }
}
