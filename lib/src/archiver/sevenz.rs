use std::fs::File;
use std::path::PathBuf;

use sevenz_rust::{SevenZArchiveEntry, SevenZMethod, SevenZMethodConfiguration, SevenZWriter};

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, ToteError};

/// 7-Zip format archiver implementation.
///
/// This archiver creates 7z archive files.
pub(super) struct SevenZArchiver {}

impl ToteArchiver for SevenZArchiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut w = match SevenZWriter::new(file) {
            Ok(writer) => writer,
            Err(e) => return Err(ToteError::Archiver(e.to_string())),
        };
        set_compression_level(&mut w, config.level);
        let mut errs = vec![];
        let mut entries = vec![];
        for tp in targets {
            for t in config.iter(tp) {
                let path = t.into_path();
                entries.push(ArchiveEntry::from(&path));
                if path.is_file() {
                    if let Err(e) = process_file(&mut w, &path, &config.path_in_archive(&path)) {
                        errs.push(e);
                    }
                }
            }
        }
        if let Err(e) = w.finish() {
            errs.push(ToteError::Archiver(e.to_string()));
        }
        if errs.is_empty() {
            Ok(entries)
        } else {
            Err(ToteError::Array(errs))
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

fn set_compression_level(szw: &mut SevenZWriter<File>, level: u8) {
    let level = match level {
        0..=4 => SevenZMethod::LZMA,
        _ => SevenZMethod::LZMA2,
    };
    szw.set_content_methods(vec![SevenZMethodConfiguration::new(level)]);
}

fn process_file(szw: &mut SevenZWriter<File>, target: &PathBuf, dest_path: &PathBuf) -> Result<()> {
    let name = &dest_path.to_str().unwrap();
    if let Err(e) = szw.push_archive_entry(
        SevenZArchiveEntry::from_path(dest_path, name.to_string()),
        Some(File::open(target).unwrap()),
    ) {
        return Err(ToteError::Archiver(e.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
    fn test_sevenz() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.7z")
                .overwrite(true)
                .build();
            let v = vec!["lib", "cli", "Cargo.toml"]
                .iter()
                .map(|s| PathBuf::from(s))
                .collect::<Vec<PathBuf>>();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{:?}", e);
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.7z");
    }
}
