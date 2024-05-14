use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::cli::{ToteError, Result};

pub(super) struct RarArchiver {
}

impl Archiver for  RarArchiver {
    fn perform(&self, _: &ArchiverOpts) -> Result<()> {
        Err(ToteError::UnsupportedFormat("only extraction support for rar".to_string()))
    }
    fn format(&self) -> Format {
        Format::Rar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use crate::verboser::create_verboser;

    #[test]
    fn test_format() {
        let archiver = RarArchiver{};
        assert_eq!(archiver.format(), Format::Rar);
    }

    #[test]
    fn test_archive() {
        let archiver = RarArchiver{};
        let opts = ArchiverOpts {
            dest: PathBuf::from("results/test.rar"),
            targets: vec![],
            overwrite: false,
            recursive: false,
            v: create_verboser(false),
        };
        let r = archiver.perform(&opts);
        assert!(r.is_err());
    }
}
