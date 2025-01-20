use std::path::PathBuf;

use crate::format;

use super::format::{find_format, Format};
use super::{Result, ToteError};

mod cab;
mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub struct ExtractorOpts {
    pub archive_file: PathBuf,
    pub dest: PathBuf,
    pub use_archive_name_dir: bool,
    pub overwrite: bool,
}

impl ExtractorOpts {
    pub fn new(archive_file: PathBuf, dest: Option<PathBuf>) -> ExtractorOpts {
        ExtractorOpts::new_with_opts(archive_file, dest, false, false)
    }

    pub fn new_with_opts(
        archive_file: PathBuf,
        dest: Option<PathBuf>,
        use_archive_name_dir: bool,
        overwrite: bool,
    ) -> ExtractorOpts {
        ExtractorOpts {
            archive_file,
            dest: dest.unwrap_or_else(|| PathBuf::from(".")),
            use_archive_name_dir,
            overwrite,
        }
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    pub fn base_dir(&self) -> PathBuf {
        if self.use_archive_name_dir {
            if let Some(stem) = self.archive_file.file_stem() {
                let dir_name = stem.to_str().unwrap();
                self.dest.join(dir_name)
            } else {
                self.dest.clone()
            }
        } else {
            self.dest.clone()
        }
    }

    pub fn destination(&self, target: &PathBuf) -> Result<PathBuf> {
        let base = self.base_dir();
        let dest = base.join(target);
        if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest.clone()))
        } else {
            Ok(dest)
        }
    }

    pub fn format(&self) -> Format {
        match format::find_format(self.archive_file.file_name()) {
            Ok(f) => f,
            Err(e) => Format::Unknown(format!("{:?}", e).to_string()),
        }
    }

    pub fn can_extract(&self) -> Result<()> {
        let dest = self.base_dir();
        if dest == PathBuf::from(".") {
            Ok(())
        } else if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest))
        } else {
            Ok(())
        }
    }
}

pub struct Extractor<'a> {
    opts: &'a ExtractorOpts,
}

impl<'a> Extractor<'a> {
    pub fn new(opts: &'a ExtractorOpts) -> Self {
        Self { opts }
    }

    pub fn perform(&self) -> Result<()> {
        let extractor = match create_extractor(&self.opts.archive_file) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        extractor.perform(&self.opts.archive_file, &self.opts)
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let extractor = match create_extractor(&self.opts.archive_file) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        extractor.list_archives(&self.opts.archive_file)
    }

    pub fn info(&self) -> String {
        let f = match format::find_format(self.opts.archive_file.file_name()) {
            Ok(f) => f,
            Err(e) => Format::Unknown(format!("{:?}", e).to_string()),
        };
        format!(
            "Format: {:?}\nFile: {:?}\nDestination: {:?}",
            f, self.opts.archive_file, self.opts.dest,
        )
    }
}

pub(crate) trait ToteExtractor {
    fn list_archives(&self, archive_file: &PathBuf) -> Result<Vec<String>>;
    fn perform(&self, archive_file: &PathBuf, opts: &ExtractorOpts) -> Result<()>;
    fn format(&self) -> Format;
}

fn create_extractor(file: &PathBuf) -> Result<Box<dyn ToteExtractor>> {
    let format = find_format(file.file_name());
    match format {
        Ok(format) => {
            return match format {
                Format::Cab => Ok(Box::new(cab::CabExtractor {})),
                Format::LHA => Ok(Box::new(lha::LhaExtractor {})),
                Format::Rar => Ok(Box::new(rar::RarExtractor {})),
                Format::SevenZ => Ok(Box::new(sevenz::SevenZExtractor {})),
                Format::Tar => Ok(Box::new(tar::TarExtractor {})),
                Format::TarBz2 => Ok(Box::new(tar::TarBz2Extractor {})),
                Format::TarGz => Ok(Box::new(tar::TarGzExtractor {})),
                Format::TarXz => Ok(Box::new(tar::TarXzExtractor {})),
                Format::TarZstd => Ok(Box::new(tar::TarZstdExtractor {})),
                Format::Zip => Ok(Box::new(zip::ZipExtractor {})),
                Format::Unknown(s) => Err(ToteError::UnknownFormat(format!(
                    "{}: unsupported format",
                    s
                ))),
            }
        }
        Err(msg) => Err(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination() {
        let target = PathBuf::from("/tmp/archive.zip");
        let opts1 = ExtractorOpts::new_with_opts(target, None, true, false);
        assert_eq!(opts1.base_dir(), PathBuf::from("./archive"));
        if let Ok(t) = opts1.destination(&PathBuf::from("text1.txt")) {
            assert_eq!(t, PathBuf::from("./archive/text1.txt"));
        }
        if let Ok(t) = opts1.destination(&PathBuf::from("text2.txt")) {
            assert_eq!(t, PathBuf::from("./archive/text2.txt"));
        }

        let target = PathBuf::from("/tmp/archive.zip");
        let opts2 = ExtractorOpts::new(target, None);
        assert_eq!(opts2.base_dir(), PathBuf::from("."));
        if let Ok(t) = opts2.destination(&PathBuf::from("./text1.txt")) {
            assert_eq!(t, PathBuf::from("./text1.txt"));
        }
    }

    #[test]
    fn test_create_extractor() {
        let e1 = create_extractor(&PathBuf::from("results/test.zip"));
        assert!(e1.is_ok());
        assert_eq!(e1.unwrap().format(), Format::Zip);

        let e2 = create_extractor(&PathBuf::from("results/test.tar"));
        assert!(e2.is_ok());
        assert_eq!(e2.unwrap().format(), Format::Tar);

        let e3 = create_extractor(&PathBuf::from("results/test.tgz"));
        assert!(e3.is_ok());
        assert_eq!(e3.unwrap().format(), Format::TarGz);

        let e4 = create_extractor(&PathBuf::from("results/test.tbz2"));
        assert!(e4.is_ok());
        assert_eq!(e4.unwrap().format(), Format::TarBz2);

        let e5 = create_extractor(&PathBuf::from("results/test.rar"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::Rar);

        let e6 = create_extractor(&PathBuf::from("results/test.tar.xz"));
        assert!(e6.is_ok());
        assert_eq!(e6.unwrap().format(), Format::TarXz);

        let e7 = create_extractor(&PathBuf::from("results/test.7z"));
        assert!(e7.is_ok());
        assert_eq!(e7.unwrap().format(), Format::SevenZ);

        let e5 = create_extractor(&PathBuf::from("results/test.lzh"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::LHA);

        let e8 = create_extractor(&PathBuf::from("results/test.unknown"));
        assert!(e8.is_err());
        if let Err(ToteError::UnknownFormat(msg)) = e8 {
            assert_eq!(msg, "test.unknown: unsupported format".to_string());
        } else {
            assert!(false);
        }
    }
}
