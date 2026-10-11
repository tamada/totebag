use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Error, Result};

/// CPIO format archiver implementation.
///
/// This archiver creates CPIO archive files.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        file: &mut BufWriter<File>,
        targets: &[PathBuf],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        let format = find_format(config.level);
        let mut builder = cpio::Builder::new(file);
        builder.set_format(format);
        let mut errs = vec![];
        let mut archived = vec![];
        for t in super::walk_targets(targets, config, &mut errs) {
            if !t.path.is_file() {
                continue;
            }
            match builder.append_path(&t.path, t.dest_path) {
                Ok(_) => archived.push(ArchiveEntry::from(&t.path)),
                Err(e) => errs.push(Error::Archiver(e.to_string())),
            }
        }
        if let Err(e) = builder.finish() {
            errs.push(Error::Archiver(e.to_string()));
        }
        Error::error_or(archived, errs)
    }

    fn enable(&self) -> bool {
        true
    }
}

fn find_format(level: u8) -> cpio::Format {
    use cpio::ByteOrder::{BigEndian, LittleEndian};
    match level {
        0..=3 => cpio::Format::Odc,
        4..=6 => cpio::Format::Newc,
        7 => cpio::Format::Crc,
        8 => cpio::Format::Bin(LittleEndian),
        9 => cpio::Format::Bin(BigEndian),
        _ => cpio::Format::Newc,
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::test_support::targets;

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
            let v = targets();
            if let Err(e) = crate::archive(&v, &config) {
                panic!("{e:?}")
            }
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.cpio");
    }
}
