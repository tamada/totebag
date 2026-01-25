use std::fs::File;
use std::path::PathBuf;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, Error};

/// LHA/LZH format archiver implementation.
///
/// Note: This archiver is not supported for creating archives,
/// only extraction is supported for LHA/LZH format.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        _: File,
        _: &[PathBuf],
        _config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        Err(Error::UnsupportedFormat(
            "only extraction support for lha".to_string(),
        ))
    }
    fn enable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_lha_archive() {
        let config = crate::ArchiveConfig::builder()
            .dest(PathBuf::from("results/test.lzh"))
            .build();
        let v: Vec<PathBuf> = Vec::new();
        let r = crate::archive(&v, &config);
        assert!(r.is_err());
        if let Err(Error::UnsupportedFormat(e)) = r {
            assert_eq!(e, "Lha: unsupported format (archiving)");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
