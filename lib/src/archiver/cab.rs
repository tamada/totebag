use std::fs::File;
use std::path::{Path, PathBuf};

use cab::{CabinetBuilder, CabinetWriter};

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, ToteError};

/// CAB (Cabinet) format archiver implementation.
///
/// This archiver creates Microsoft Cabinet archive files.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: File,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let mut errs = vec![];
        let mut entries = vec![];
        let mut builder = CabinetBuilder::new();
        let ctype = compression_type(config.level);
        let folder = builder.add_folder(ctype);
        let list = collect_entries(targets, config);
        for path in list.iter() {
            entries.push(ArchiveEntry::from(path));
            folder.add_file(config.path_in_archive(path).to_str().unwrap());
        }
        let mut writer = match builder.build(file) {
            Ok(w) => w,
            Err(e) => return Err(ToteError::Archiver(e.to_string())),
        };
        for path in list.iter() {
            if let Err(e) = write_entry(&mut writer, path) {
                errs.push(e);
            }
        }
        match writer.finish() {
            Ok(_) => Ok(entries),
            Err(e) => Err(ToteError::Archiver(e.to_string())),
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

fn compression_type(level: u8) -> cab::CompressionType {
    match level {
        0 => cab::CompressionType::None,
        _ => cab::CompressionType::MsZip,
    }
}

fn write_entry(writer: &mut CabinetWriter<File>, path: &Path) -> Result<()> {
    match (File::open(path), writer.next_file()) {
        (Ok(mut reader), Ok(Some(mut w))) => match std::io::copy(&mut reader, &mut w) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToteError::IO(e)),
        },
        (_, Ok(None)) => Err(ToteError::Archiver("cab writer error".to_string())),
        (Err(e1), Err(e2)) => Err(ToteError::Array(vec![
            ToteError::IO(e1),
            ToteError::Fatal(Box::new(e2)),
        ])),
        (Err(e), _) => Err(ToteError::IO(e)),
        (_, Err(e)) => Err(ToteError::Archiver(e.to_string())),
    }
}

fn collect_entries<P: AsRef<Path>>(targets: &[P], config: &crate::ArchiveConfig) -> Vec<PathBuf> {
    let mut r = vec![];
    for path in targets {
        for entry in config.iter(path) {
            let path = entry.into_path();
            if path.is_file() {
                r.push(path)
            }
        }
    }
    r
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
    fn test_archive() {
        run_test(|| {
            let config = crate::ArchiveConfig::builder()
                .dest("results/test.cab")
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
        let _ = std::fs::remove_file("results/test.cab");
    }
}
