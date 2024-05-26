use std::path::PathBuf;

use crate::cli::{Result, ToteError};
use crate::format::{find_format, Format};
use crate::verboser::{create_verboser, Verboser};
use crate::CliOpts;

mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub struct ExtractorOpts {
    pub dest: PathBuf,
    pub use_archive_name_dir: bool,
    pub overwrite: bool,
    pub v: Box<dyn Verboser>,
}

impl ExtractorOpts {
    pub fn new(opts: &CliOpts) -> ExtractorOpts {
        let d = opts.output.clone();
        ExtractorOpts {
            dest: d.unwrap_or_else(|| PathBuf::from(".")),
            use_archive_name_dir: opts.to_archive_name_dir,
            overwrite: opts.overwrite,
            v: create_verboser(opts.verbose),
        }
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    pub fn destination(&self, target: &PathBuf) -> PathBuf {
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

pub trait Extractor {
    fn list_archives(&self, archive_file: PathBuf) -> Result<Vec<String>>;
    fn perform(&self, archive_file: PathBuf, opts: &ExtractorOpts) -> Result<()>;
    fn format(&self) -> Format;
}

pub fn create_extractor(file: &PathBuf) -> Result<Box<dyn Extractor>> {
    let format = find_format(file.file_name());
    match format {
        Ok(format) => {
            return match format {
                Format::Zip => Ok(Box::new(zip::ZipExtractor {})),
                Format::Rar => Ok(Box::new(rar::RarExtractor {})),
                Format::Tar => Ok(Box::new(tar::TarExtractor {})),
                Format::TarGz => Ok(Box::new(tar::TarGzExtractor {})),
                Format::TarBz2 => Ok(Box::new(tar::TarBz2Extractor {})),
                Format::TarXz => Ok(Box::new(tar::TarXzExtractor {})),
                Format::TarZstd => Ok(Box::new(tar::TarZstdExtractor {})),
                Format::LHA => Ok(Box::new(lha::LhaExtractor {})),
                Format::SevenZ => Ok(Box::new(sevenz::SevenZExtractor {})),
                Format::Unknown(s) => Err(ToteError::UnknownFormat(format!(
                    "{}: unsupported format",
                    s
                ))),
            }
        }
        Err(msg) => Err(msg),
    }
}

pub fn extractor_info(
    extractor: &Box<dyn Extractor>,
    target: &PathBuf,
    opts: &ExtractorOpts,
) -> String {
    format!(
        "Format: {:?}\nFile: {:?}\nDestination: {:?}",
        extractor.format(),
        target,
        opts.dest,
    )
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
            v: create_verboser(false),
        };
        let target = PathBuf::from("/tmp/archive.zip");
        assert_eq!(opts1.destination(&target), PathBuf::from("./archive"));

        let opts2 = ExtractorOpts {
            dest: PathBuf::from("."),
            use_archive_name_dir: false,
            overwrite: false,
            v: create_verboser(false),
        };
        let target = PathBuf::from("/tmp/archive.zip");
        assert_eq!(opts2.destination(&target), PathBuf::from("."));
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
