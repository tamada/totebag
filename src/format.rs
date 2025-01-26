use std::fmt::Display;
use std::path::Path;

use super::{Result, ToteError};

/// return `true` if all of `args` is an acceptable archive file name for totebag.
/// ```
/// args.iter().all(is_archive_file)
/// ```
pub fn is_all_args_archives<P: AsRef<Path>>(args: &[P]) -> bool {
    args.iter().all(is_archive_file)
}

/// returns `true`` when the given path is an acceptable archive file name for totebag.
pub fn is_archive_file<P: AsRef<Path>>(arg: P) -> bool {
    let arg = arg.as_ref();
    let name = arg.to_str().unwrap().to_lowercase();
    for (_, ext) in exts().iter() {
        if name.ends_with(ext) {
            return true;
        }
    }
    false
}

/// Find the format of the given file name.
/// If the given file name has an unknown extension for totebag, it returns an `Err(ToteErro::Unknown)`.
pub fn find_format<P: AsRef<Path>>(path: P) -> Result<Format> {
    match path.as_ref().file_name() {
        Some(file_name) => {
            let name = file_name.to_str().unwrap().to_lowercase();
            for ext in exts().iter() {
                if name.ends_with(&ext.1) {
                    return Ok(ext.0.clone());
                }
            }
            Err(ToteError::UnknownFormat(
                file_name.to_str().unwrap().to_string(),
            ))
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
        if let Err(e) = find_format("hoge.unknown") {
            if let ToteError::UnknownFormat(s) = e {
                assert_eq!(s, "hoge.unknown".to_string());
            } else {
                assert!(false);
            }
        }
        if let Ok(f) = find_format("hoge.zip") {
            assert_eq!(f, Format::Zip);
            assert_eq!(f.to_string(), "Zip".to_string());
        }
        if let Ok(f) = find_format("hoge.tar") {
            assert_eq!(f, Format::Tar);
            assert_eq!(f.to_string(), "Tar".to_string());
        }
        if let Ok(f) = find_format("hoge.rar") {
            assert_eq!(f, Format::Rar);
            assert_eq!(f.to_string(), "Rar".to_string());
        }
        if let Ok(f) = find_format("hoge.tar.gz") {
            assert_eq!(f, Format::TarGz);
            assert_eq!(f.to_string(), "TarGz".to_string());
        }
        if let Ok(f) = find_format("hoge.tar.bz2") {
            assert_eq!(f, Format::TarBz2);
            assert_eq!(f.to_string(), "TarBz2".to_string());
        }
        if let Ok(f) = find_format("hoge.tar.xz") {
            assert_eq!(f, Format::TarXz);
            assert_eq!(f.to_string(), "TarXz".to_string());
        }
        if let Ok(f) = find_format("hoge.7z") {
            assert_eq!(f, Format::SevenZ);
            assert_eq!(f.to_string(), "SevenZ".to_string());
        }
        if let Err(e) = find_format(".") {
            if let ToteError::NoArgumentsGiven = e {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_is_all_args_archives() {
        assert!(is_all_args_archives(&[
            "test.zip",
            "test.tar",
            "test.tar.gz",
            "test.tgz",
            "test.tar.bz2",
            "test.tbz2",
            "test.rar",
        ]));
    }
}
