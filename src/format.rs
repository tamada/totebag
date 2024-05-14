use std::ffi::OsStr;
use std::fmt::Display;
use crate::cli::{ToatError, Result};

pub fn find_format(file_name: Option<&OsStr>) -> Result<Format> {
    match file_name {
        Some(file_name) => {
            let name = file_name.to_str().unwrap().to_lowercase();
            if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
                return Ok(Format::TarGz);
            } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
                return Ok(Format::TarBz2);
            } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
                return Ok(Format::TarXz);
            } else if name.ends_with(".7z") {
                return Ok(Format::SevenZ);
            } else if name.ends_with(".tar") {
                return Ok(Format::Tar);
            } else if name.ends_with(".rar") {
                return Ok(Format::Rar);
            } else if name.ends_with(".zip") || name.ends_with(".jar") || name.ends_with(".war") || name.ends_with(".ear") {
                return Ok(Format::Zip);
            } else {
                return Err(ToatError::UnknownFormat(name.to_string()));
            }
        }
        None => Err(ToatError::NoArgumentsGiven),
    }
}


#[derive(Debug, PartialEq)]
pub enum Format {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    SevenZ,
    Rar,
    Unknown(String),
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Zip => write!(f, "Zip"),
            Format::Tar => write!(f, "Tar"),
            Format::TarGz => write!(f, "TarGz"),
            Format::TarBz2 => write!(f, "TarBz2"),
            Format::TarXz => write!(f, "TarXz"),
            Format::SevenZ => write!(f, "SevenZ"),
            Format::Rar => write!(f, "Rar"),
            Format::Unknown(s) => write!(f, "{}: unknown format", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert!(find_format(None).is_err());
        if let Ok(f) = find_format(Some(OsStr::new("hoge.zip"))) {
            assert_eq!(f, Format::Zip);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.unknown"))) {
            assert_eq!(f.to_string(), "hoge.unknown: unknown format".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar"))) {
            assert_eq!(f, Format::Tar);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.rar"))) {
            assert_eq!(f, Format::Rar);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.gz"))) {
            assert_eq!(f, Format::TarGz);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.bz2"))) {
            assert_eq!(f, Format::TarBz2);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.xz"))) {
            assert_eq!(f, Format::TarXz);
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.7z"))) {
            assert_eq!(f, Format::SevenZ);
        }
    }
}