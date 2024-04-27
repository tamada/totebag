use crate::archiver::{Archiver, InOut, Format};
use crate::cli::{ToatError, Result};

pub struct RarArchiver {
}

impl Archiver for  RarArchiver {
    fn perform(&self, inout: InOut) -> Result<()> {
        Err(ToatError::UnknownError("not implement yet".to_string()))
    }
    fn format(&self) -> Format {
        Format::Rar
    }
}
