use std::fs::File;

use crate::archiver::{ArchiveEntry, Targets, ToteArchiver};
use crate::{Result, ToteError};

pub(super) struct RarArchiver {}

impl ToteArchiver for RarArchiver {
    fn perform(&self, _: File, _: Targets) -> Result<Vec<ArchiveEntry>> {
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
    use crate::archiver::Archiver;
    use crate::ToteError;
    use std::path::PathBuf;

    #[test]
    fn test_rar_archive() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.rar"))
            .targets(vec![])
            .build();

        let r = archiver.perform();
        assert!(r.is_err());
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "Rar: not support archiving");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
