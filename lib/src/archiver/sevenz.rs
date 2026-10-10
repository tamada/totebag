use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, Write};
use std::path::{Path, PathBuf};

use sevenz_rust2::{ArchiveEntry, ArchiveWriter, EncoderConfiguration, EncoderMethod};

use crate::archiver::{ArchiveEntry as ToteArchiveEntry, ToteArchiver};
use crate::{Error, Result};

/// 7-Zip format archiver implementation.
///
/// This archiver creates 7z archive files.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ToteArchiveEntry>> {
        let mut w = match ArchiveWriter::new(file) {
            Ok(writer) => writer,
            Err(e) => return Err(Error::Archiver(e.to_string())),
        };
        let methods = compression_level(config.level);
        w.set_content_methods(methods);
        let mut errs = vec![];
        let mut entries = vec![];
        for tp in targets {
            for t in config.iter(tp) {
                let path = t.into_path();
                entries.push(ToteArchiveEntry::from(&path));
                if path.is_file()
                    && let Err(e) = process_file(&mut w, &path, &config.path_in_archive(&path))
                {
                    errs.push(e);
                }
            }
        }
        if let Err(e) = w.finish() {
            errs.push(Error::Archiver(e.to_string()));
        }
        Error::error_or(entries, errs)
    }

    fn enable(&self) -> bool {
        true
    }
}

fn compression_level(level: u8) -> Vec<EncoderConfiguration> {
    let method = match level {
        0..=4 => EncoderMethod::LZMA,
        _ => EncoderMethod::LZMA2,
    };
    vec![EncoderConfiguration::new(method)]
}

fn process_file<W: Write + Seek>(
    szw: &mut ArchiveWriter<W>,
    target: &Path,
    dest_path: &Path,
) -> Result<()> {
    let name = dest_path.to_string_lossy().into_owned();
    let source = File::open(target).map_err(Error::IO)?;
    let buf = BufReader::new(source);
    szw.push_archive_entry(ArchiveEntry::from_path(dest_path, name), Some(buf))
        .map(|_| ())
        .map_err(|e| Error::Archiver(e.to_string()))
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
    fn test_sevenz() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.7z")
                .overwrite(true)
                .build();
            let v = targets();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{e:?}");
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.7z");
    }
}
