use std::{ffi::OsStr, path::PathBuf};

pub trait Archiver {
    fn perform(&self, inout: InOut) -> Result<(), String>;
    fn format(&self) -> Format;
}

struct ZipArchiver {
}
struct TarArchiver {
}
struct TarGzArchiver {
}
struct TarBz2Archiver {
}
struct RarArchiver {
}

impl Archiver for  ZipArchiver {
    fn perform(&self, inout: InOut) -> Result<(), String> {
        Err("not implement yet".to_string())
    }
    fn format(&self) -> Format {
        Format::Zip
    }
}
impl Archiver for  TarArchiver {
    fn perform(&self, inout: InOut) -> Result<(), String> {
        Err("not implement yet".to_string())
    }
    fn format(&self) -> Format {
        Format::Tar
    }
}
impl Archiver for TarGzArchiver{
    fn perform(&self, inout: InOut) -> Result<(), String> {
        Err("not implement yet".to_string())
    }
    fn format(&self) -> Format {
        Format::TarGz
    }
}
impl Archiver for  TarBz2Archiver {
    fn perform(&self, inout: InOut) -> Result<(), String> {
        Err("not implement yet".to_string())
    }
    fn format(&self) -> Format {
        Format::TarBz2
    }
}
impl Archiver for  RarArchiver {
    fn perform(&self, inout: InOut) -> Result<(), String> {
        Err("not implement yet".to_string())
    }
    fn format(&self) -> Format {
        Format::Rar
    }
}

pub fn create_archiver(dest: PathBuf) -> Result<Box<dyn Archiver>, String> {
    let format = find_format(dest.file_name());
    match format {
        Ok(format) => {
            return match format {
                Format::Zip => Ok(Box::new(ZipArchiver{})),
                Format::Tar => Ok(Box::new(TarArchiver{})),
                Format::TarGz => Ok(Box::new(TarGzArchiver{})),
                Format::TarBz2 => Ok(Box::new(TarBz2Archiver{})),
                Format::Rar => Ok(Box::new(RarArchiver{})),
                _ => Err("unsupported format".to_string()),
            }
        }
        Err(msg) => Err(msg),
    }
}

pub fn archiver_info(archiver: Box<dyn Archiver>, inout: InOut) -> String {
    format!(
        "Format: {:?}\nDestination: {:?}\nTargets: {:?}",
        archiver.format(),
        inout.destination(),
        inout.targets().iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>().join(", ")
    )
}

pub struct InOut {
    dest: PathBuf,
    targets: Vec<PathBuf>,
}

impl InOut {
    pub fn new(dest: PathBuf, targets: Vec<PathBuf>) -> Self {
        InOut { dest, targets }
    }
    pub fn targets(&self) -> Vec<PathBuf> {
        self.targets.clone()
    }
    pub fn destination(&self) -> &PathBuf {
        &self.dest
    }
}

fn find_format(file_name: Option<&OsStr>) -> Result<Format, String> {
    match file_name {
        Some(file_name) => {
            let name = file_name.to_str().unwrap().to_lowercase();
            if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
                return Ok(Format::TarGz);
            } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
                return Ok(Format::TarBz2);
            } else if name.ends_with(".tar") {
                return Ok(Format::Tar);
            } else if name.ends_with(".rar") {
                return Ok(Format::Rar);
            } else if name.ends_with(".zip") {
                return Ok(Format::Zip);
            } else {
                return Ok(Format::Unknown);
            }
        }
        None => Err("no file name provided".to_string()),
    }
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    Rar,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert!(find_format(None).is_err());
        assert_eq!(find_format(Some(OsStr::new("hoge.zip"))), Ok(Format::Zip));
        assert_eq!(find_format(Some(OsStr::new("hoge.unknown"))), Ok(Format::Unknown));
        assert_eq!(find_format(Some(OsStr::new("hoge.tar"))), Ok(Format::Tar));
        assert_eq!(find_format(Some(OsStr::new("hoge.rar"))), Ok(Format::Rar));
        assert_eq!(find_format(Some(OsStr::new("hoge.tar.gz"))), Ok(Format::TarGz));
        assert_eq!(find_format(Some(OsStr::new("hoge.tar.bz2"))), Ok(Format::TarBz2));
    }
}