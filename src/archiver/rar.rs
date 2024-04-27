use crate::archiver::{Archiver, InOut, Format};
use crate::cli::{ToatError, Result};

pub struct RarArchiver {
}

impl Archiver for  RarArchiver {
    fn perform(&self, _: InOut) -> Result<()> {
        Err(ToatError::UnsupportedFormat("only extraction support for rar".to_string()))
    }
    fn format(&self) -> Format {
        Format::Rar
    }
}
