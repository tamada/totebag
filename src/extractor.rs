use std::path::PathBuf;

use super::{Result, ToteError};
use super::format::{find_format, Format};

mod cab;
mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub struct ExtractorOpts {
    pub dest: PathBuf,
    pub use_archive_name_dir: bool,
    pub overwrite: bool,
}

impl ExtractorOpts {
    pub fn new(dest: Option<PathBuf>) -> ExtractorOpts {
        ExtractorOpts::new_with_opts(dest, false, false)
    }

    pub fn new_with_opts(dest: Option<PathBuf>, use_archive_name_dir: bool, overwrite: bool) -> ExtractorOpts {
        ExtractorOpts {
            dest: dest.unwrap_or_else(|| PathBuf::from(".")),
            use_archive_name_dir,
            overwrite,
        }
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    pub fn destination(&self, target: &PathBuf) -> Result<PathBuf> {
        let dest = self.destination_file(target);
        if dest.exists() && !self.overwrite {
            if dest == PathBuf::from(".") {
                Ok(dest)
            } else {
                Err(ToteError::FileExists(dest.clone()))
            }
        } else {
            Ok(dest)
        }
    }

    fn destination_file(&self, target: &PathBuf) -> PathBuf {
        if self.use_archive_name_dir {
            let file_name = target.file_name().unwrap().to_str().unwrap();
            let ext = target.extension().unwrap().to_str().unwrap();
            let dir_name = file_name
                .trim_end_matches(ext)
                .trim_end_matches(".")
                .to_string();
            self.dest.join(dir_name)
        } else {
            self.dest.clone()
        }
    }
}


pub struct Extractor<'a> {
    archive_file: PathBuf,
    opts: &'a ExtractorOpts,
    extractor: Box<dyn ToteExtractor>,
}

impl<'a> Extractor<'a> {
    pub fn new(archive_file: PathBuf, opts: &'a ExtractorOpts) -> Result<Self> {
        match create_extractor(&archive_file) {
            Ok(extractor) => Ok(Self {
                archive_file,
                opts,
                extractor,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn format(&self) -> Format {
        self.extractor.format()
    }

    pub fn perform(&self) -> Result<()> {
        self.extractor.perform(&self.archive_file, &self.opts)
    }

    pub fn can_extract(&self) -> Result<()> {
        let dest = self.target_dir();
        if dest == PathBuf::from(".") {
            Ok(())
        } else {
            if dest.exists() && dest.is_dir() && self.opts.overwrite {
                Err(ToteError::DirExists(dest))
            } else if dest.is_file() {
                Err(ToteError::FileExists(dest))
            } else {
                Ok(())
            }
        }
    }

    pub fn list(&self) -> Result<Vec<String>> {
        self.extractor.list_archives(&self.archive_file)
    }

    pub fn info(&self) -> String {
        format!(
            "Format: {:?}\nFile: {:?}\nDestination: {:?}",
            self.extractor.format(),
            self.archive_file,
            self.opts.dest,
        )
    }

    pub fn target_dir(&self) -> PathBuf {
        if self.opts.use_archive_name_dir {
            let base = self.opts.dest.clone();
            if let Some(file_name) = self.archive_file.file_stem() {
                let dir_name = file_name.to_str().unwrap();
                self.opts.dest.join(dir_name)
            } else {
                base
            }
        } else {
            self.opts.dest.clone()
        }
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
        let opts1 = ExtractorOpts {
            dest: PathBuf::from("."),
            use_archive_name_dir: true,
            overwrite: false,
        };
        let target = PathBuf::from("/tmp/archive.zip");

        if let Ok(t) = opts1.destination(&target) {
            assert_eq!(t, PathBuf::from("./archive"));
        }

        let opts2 = ExtractorOpts {
            dest: PathBuf::from("."),
            use_archive_name_dir: false,
            overwrite: false,
        };
        let target = PathBuf::from("/tmp/archive.zip");
        if let Ok(t) = opts2.destination(&target) {
            assert_eq!(t, PathBuf::from("."));
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

        let e8 = create_extractor(&PathBuf::from("results/test.unknown"));
        assert!(e8.is_err());
        if let Err(ToteError::UnknownFormat(msg)) = e8 {
            assert_eq!(msg, "test.unknown: unsupported format".to_string());
        } else {
            assert!(false);
        }
    }
}
