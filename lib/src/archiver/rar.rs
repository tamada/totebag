use std::fs::File;
use std::path::PathBuf;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, ToteError};

/// RAR format archiver implementation.
///
/// Note: This archiver is not supported for creating archives,
/// only extraction is supported for RAR format.
pub(super) struct Archiver {}

impl ToteArchiver for Archiver {
    fn perform(
        &self,
        _: File,
        _: &[PathBuf],
        _config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        Err(ToteError::UnsupportedFormat(
            "only extraction support for rar".to_string(),
        ))
    }

    fn enable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::ArchiveConfig;
    use crate::ToteError;
    use std::path::PathBuf;

    #[test]
    fn test_rar_archive() {
        let config = ArchiveConfig::builder().dest("results/test.rar").build();
        let v = Vec::<PathBuf>::new();

        let r = crate::archive(&v, &config);
        assert!(r.is_err());
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "Rar: unsupported format (archiving)");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
