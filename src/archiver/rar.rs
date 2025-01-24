use std::fs::File;

use crate::archiver::{ArchiverOpts, Format, ToteArchiver};
use crate::{Result, ToteError};

use super::TargetPath;

pub(super) struct RarArchiver {}

impl ToteArchiver for RarArchiver {
    fn perform_impl(&self, _: File, _: Vec<TargetPath>, _: &ArchiverOpts) -> Result<()> {
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
        let opts = ArchiverOpts::create(None, false, false, vec![]);
        let archiver = Archiver::new(PathBuf::from("results/test.rar"), vec![], &opts);

        let r = archiver.perform();
        assert!(r.is_err());
    }

    #[test]
    fn test_lha_archiver() {
        let archiver = RarArchiver {};
        assert_eq!(archiver.format(), Format::Rar);
        let file = File::create("results/test.rar").unwrap();
        let r = archiver.perform_impl(
            file,
            vec![],
            &ArchiverOpts::create(None, false, false, vec![]),
        );
        if let Err(ToteError::UnsupportedFormat(e)) = r {
            assert_eq!(e, "only extraction support for rar");
        } else {
            panic!("unexpected result: {:?}", r);
        }
    }
}
