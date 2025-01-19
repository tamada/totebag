use std::fs::File;

use crate::archiver::{ToteArchiver, ArchiverOpts, Format};
use crate::{Result, ToteError};

use super::TargetPath;

pub(super) struct LhaArchiver {}

impl ToteArchiver for LhaArchiver {
    fn perform(&self, _: File, _: Vec<TargetPath>, _: &ArchiverOpts) -> Result<()> {
        Err(ToteError::UnsupportedFormat(
            "only extraction support for lha".to_string(),
        ))
    }
    fn enable(&self) -> bool {
        false
    }
    fn format(&self) -> Format {
        Format::LHA
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::Archiver;

    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_format() {
        let archiver = LhaArchiver {};
        assert_eq!(archiver.format(), Format::LHA);
    }

    #[test]
    fn test_archive() {
        let opts = ArchiverOpts::create(
            None, false, false, vec![]);
        let archiver = Archiver::new(
            PathBuf::from("results/test.lzh"),
            vec![],
            opts).unwrap();

        let r = archiver.perform();
        assert!(r.is_err());
    }
}
