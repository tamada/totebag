use crate::archiver::{Archiver, Format, ArchiverOpts};
use crate::cli::{ToatError, Result};

pub(super) struct RarArchiver {
}

impl Archiver for  RarArchiver {
    fn perform(&self, _: ArchiverOpts) -> Result<()> {
        Err(ToatError::UnsupportedFormat("only extraction support for rar".to_string()))
    }
    fn format(&self) -> Format {
        Format::Rar
    }
}
