use crate::archiver::{Archiver, ArchiverOpts, Format};
use crate::cli::{Result, ToteError};

pub(super) struct LhaArchiver {}

impl Archiver for LhaArchiver {
    fn perform(&self, _: &ArchiverOpts) -> Result<()> {
        Err(ToteError::UnsupportedFormat(
            "only extraction support for lha".to_string(),
        ))
    }
    fn format(&self) -> Format {
        Format::LHA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::verboser::create_verboser;
    use std::path::PathBuf;

    #[test]
    fn test_format() {
        let archiver = LhaArchiver {};
        assert_eq!(archiver.format(), Format::LHA);
    }

    #[test]
    fn test_archive() {
        let archiver = LhaArchiver {};
        let opts = ArchiverOpts {
            dest: PathBuf::from("results/test.lzh"),
            targets: vec![],
            base_dir: PathBuf::from("."),
            overwrite: false,
            recursive: false,
            v: create_verboser(false),
        };
        let r = archiver.perform(&opts);
        assert!(r.is_err());
    }
}
