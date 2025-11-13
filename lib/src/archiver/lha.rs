use std::fs::File;
use std::path::PathBuf;

use crate::archiver::{ArchiveEntry, ToteArchiver};
use crate::{Result, ToteError};

pub(super) struct LhaArchiver {}

impl ToteArchiver for LhaArchiver {
    fn perform(
        &self,
        _: File,
        _: &Vec<PathBuf>,
        _config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>> {
        Err(ToteError::UnsupportedFormat(
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
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "Lha: unsupported format (archiving)");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
