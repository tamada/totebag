use std::fs::File;

use crate::archiver::{ArchiverOpts, Format, ToteArchiver};
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
        let opts = ArchiverOpts::create(None, false, false, vec![]);
        let archiver = Archiver::new(PathBuf::from("results/test.lzh"), vec![], &opts);

        let r = archiver.perform();
        assert!(r.is_err());
    }

    #[test]
    fn test_lha_archiver() {
        let archiver = LhaArchiver {};
        assert_eq!(archiver.format(), Format::LHA);
        let file = File::create("results/test.lzh").unwrap();
        let r = archiver.perform(
            file,
            vec![],
            &ArchiverOpts::create(None, false, false, vec![]),
        );
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "only extraction support for lha");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
