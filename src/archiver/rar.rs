use std::fs::File;

use crate::archiver::{Format, ToteArchiver};
use crate::{Result, ToteError};

use super::TargetPath;

pub(super) struct RarArchiver {}

impl ToteArchiver for RarArchiver {
    fn perform(&self, _: File, _: Vec<TargetPath>) -> Result<()> {
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
