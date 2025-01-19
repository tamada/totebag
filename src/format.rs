use std::{ffi::OsStr, path::PathBuf};
use std::fmt::Display;

use super::{Result, ToteError};

pub fn is_all_args_archives(args: &[PathBuf]) -> bool {
    args.iter().all(is_archive_file)
}

pub fn is_archive_file(arg: &PathBuf) -> bool {
    let name = arg.to_str().unwrap().to_lowercase();
    for (_, ext) in exts().iter() {
        if name.ends_with(ext) {
            return true
        }
    }
    return false
}

pub fn find_format(file_name: Option<&OsStr>) -> Result<Format> {
    match file_name {
        Some(file_name) => {
            let name = file_name.to_str().unwrap().to_lowercase();
            for ext in exts().iter() {
                if name.ends_with(&ext.1) {
                    return Ok(ext.0.clone());
                }
            }
            return Ok(Format::Unknown(file_name.to_str().unwrap().to_string()));
        }
        None => Err(ToteError::NoArgumentsGiven),
    }
}

fn exts() -> Vec<(Format, String)> {
    vec![
        (Format::Cab, String::from(".cab")),
        (Format::LHA, String::from(".lha")),
        (Format::LHA, String::from(".lzh")),
        (Format::SevenZ, String::from(".7z")),
        (Format::Rar, String::from(".rar")),
        (Format::Tar, String::from(".tar")),
        (Format::TarGz, String::from(".tar.gz")),
        (Format::TarGz, String::from(".tgz")),
        (Format::TarBz2, String::from(".tar.bz2")),
        (Format::TarBz2, String::from(".tbz2")),
        (Format::TarXz, String::from(".tar.xz")),
        (Format::TarXz, String::from(".txz")),
        (Format::TarZstd, String::from(".tar.zst")),
        (Format::TarZstd, String::from(".tzst")),
        (Format::Zip, String::from(".zip")),
        (Format::Zip, String::from(".jar")),
        (Format::Zip, String::from(".war")),
        (Format::Zip, String::from(".ear")),
    ]
}

#[derive(Debug, PartialEq, Clone)]
pub enum Format {
    Cab,
    LHA,
    SevenZ,
    Rar,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZstd,
    Zip,
    Unknown(String),
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Cab => write!(f, "Cab"),
            Format::LHA => write!(f, "LHA"),
            Format::SevenZ => write!(f, "SevenZ"),
            Format::Rar => write!(f, "Rar"),
            Format::Tar => write!(f, "Tar"),
            Format::TarBz2 => write!(f, "TarBz2"),
            Format::TarGz => write!(f, "TarGz"),
            Format::TarXz => write!(f, "TarXz"),
            Format::TarZstd => write!(f, "TarZstd"),
            Format::Zip => write!(f, "Zip"),
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
            assert_eq!(f.to_string(), "Zip".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.unknown"))) {
            assert_eq!(f.to_string(), "hoge.unknown: unknown format".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar"))) {
            assert_eq!(f, Format::Tar);
            assert_eq!(f.to_string(), "Tar".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.rar"))) {
            assert_eq!(f, Format::Rar);
            assert_eq!(f.to_string(), "Rar".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.gz"))) {
            assert_eq!(f, Format::TarGz);
            assert_eq!(f.to_string(), "TarGz".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.bz2"))) {
            assert_eq!(f, Format::TarBz2);
            assert_eq!(f.to_string(), "TarBz2".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.tar.xz"))) {
            assert_eq!(f, Format::TarXz);
            assert_eq!(f.to_string(), "TarXz".to_string());
        }
        if let Ok(f) = find_format(Some(OsStr::new("hoge.7z"))) {
            assert_eq!(f, Format::SevenZ);
            assert_eq!(f.to_string(), "SevenZ".to_string());
        }
        if let Err(e) = find_format(None) {
            if let ToteError::NoArgumentsGiven = e {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_is_all_args_archives() {
        assert!(is_all_args_archives(&[PathBuf::from("test.zip"), PathBuf::from("test.tar"), PathBuf::from("test.tar.gz"), PathBuf::from("test.tgz"), PathBuf::from("test.tar.bz2"), PathBuf::from("test.tbz2"), PathBuf::from("test.rar")]));
    }
}
