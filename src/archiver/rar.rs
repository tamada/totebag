use std::fs::File;

use crate::archiver::{ToteArchiver, ArchiverOpts, Format};
use crate::{Result, ToteError};

use super::TargetPath;

pub(super) struct RarArchiver {}

impl ToteArchiver for RarArchiver {
    fn perform(&self, _: File, _: Vec<TargetPath>, _: &ArchiverOpts) -> Result<()> {
        Err(ToteError::UnsupportedFormat(
            "only extraction support for rar".to_string(),
        ))
    }
    fn enable(&self) -> bool {
        false
    }
    
    fn format(&self) -> Format {
        Format::Rar
    }
}

#[cfg(test)]
mod tests {
    use crate::archiver::Archiver;

    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_format() {
        let archiver = RarArchiver {};
        assert_eq!(archiver.format(), Format::Rar);
    }

    #[test]
    fn test_archive() {
        let opts = ArchiverOpts::create(
            None, false, false, vec![]);
        let archiver = Archiver::new(
            PathBuf::from("results/test.rar"),
            vec![],
            opts).unwrap();

        let r = archiver.perform();
        assert!(r.is_err());
    }
}
