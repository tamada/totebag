use std::fs::File;

use crate::archiver::ToteArchiver;
use crate::{Result, ToteError};

use super::TargetPath;

pub(super) struct LhaArchiver {}

impl ToteArchiver for LhaArchiver {
    fn perform(&self, _: File, _: Vec<TargetPath>) -> Result<()> {
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
    use crate::archiver::Archiver;

    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_lha_archive() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.lzh"))
            .targets(vec![])
            .build();
        let r = archiver.perform();
        assert!(r.is_err());
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "Lha: not support archiving");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
