/*!
 * This module provides the extractor for the archive file.
 * The supported formats are `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
 *
 * # Example: listing the entries in the archive file
 *
 * ```
 * let file = PathBuf::from("testdata/test.zip");
 * let extractor = Extractor::builder()
 *     .archive_file(file)
 *     .build();
 * match extractor.list() {
 *     Ok(entries) => println!("{:?}", entries),
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 *
 * # Example: extracting the archive file
 *
 * The destination for extraction is the current directory in the following example.
 *
 * ```
 * let extractor = Extractor::builder()
 *     .archive_file(PathBuf::From("testdata/test.zip"))
 *     .build();
 * match extractor.perform(&opts) {
 *     Ok(r) => println!("{:?}", r),
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 */
use chrono::NaiveDateTime;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use typed_builder::TypedBuilder;

use super::format::{find_format, Format};
use super::{Result, ToteError};

mod cab;
mod lha;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// This struct represents an entry in the archive file.
#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub compressed_size: Option<u64>,
    pub original_size: Option<u64>,
    pub unix_mode: Option<u32>,
    pub date: Option<NaiveDateTime>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Entry {
    pub fn new(
        name: String,
        compressed_size: Option<u64>,
        original_size: Option<u64>,
        unix_mode: Option<u32>,
        date: Option<NaiveDateTime>,
    ) -> Self {
        Self {
            name,
            compressed_size,
            original_size,
            unix_mode,
            date,
        }
    }
}

pub struct PathUtils<'a> {
    e: &'a Extractor,
}

impl PathUtils<'_> {
    pub fn base_dir(&self) -> PathBuf {
        self.e.base_dir()
    }

    fn destination<P: AsRef<Path>>(&self, target: P) -> Result<PathBuf> {
        self.e.destination(target)
    }
}

/// This struct represents the extractor for the archive file.
#[derive(Debug, TypedBuilder)]
pub struct Extractor {
    /// The archive file for extraction.
    pub archive_file: PathBuf,

    /// The destination directory for the extracted files.
    /// The default is the current directory.
    #[builder(default = PathBuf::from("."), setter(into))]
    pub destination: PathBuf,

    /// If true, the destination directory is the result of `destination` joined the stem of `archive_file`.
    /// For example, if `archive_file` is `/tmp/test.zip`, the `destination` is `/tmp/archive`,
    /// the resultant destination directory is `/tmp/archive/test`.
    #[builder(default = false)]
    pub use_archive_name_dir: bool,

    /// If true, it overwrite the existing file in the destination directory.
    #[builder(default = false)]
    pub overwrite: bool,
}

impl Extractor {
    /// Returns the entries in the archive file.
    pub fn list(&self) -> Result<Vec<Entry>> {
        let extractor = match create(&self.archive_file) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        extractor.list(self.archive_file.clone())
    }

    /// Execute extraction of the archive file.
    pub fn perform(&self) -> Result<()> {
        let extractor = match create(&self.archive_file) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        match self.can_extract() {
            Ok(_) => extractor.perform(self.archive_file.clone(), PathUtils { e: self }),
            Err(e) => Err(e),
        }
    }

    /// Returns the information of the extractor.
    pub fn info(&self) -> String {
        format!(
            "Format: {:?}\nFile: {:?}\nDestination: {:?}",
            find_format(&self.archive_file).unwrap(),
            self.archive_file,
            self.destination,
        )
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    fn base_dir(&self) -> PathBuf {
        if self.use_archive_name_dir {
            if let Some(stem) = self.archive_file.file_stem() {
                self.destination.join(stem)
            } else {
                self.destination.clone()
            }
        } else {
            self.destination.clone()
        }
    }

    /// Return the path of the `target` file for output.
    fn destination<P: AsRef<Path>>(&self, target: P) -> Result<PathBuf> {
        let base = self.base_dir();
        let dest = base.join(target);
        if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest.clone()))
        } else {
            Ok(dest)
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

/// The trait for extracting the archive file.
pub(crate) trait ToteExtractor {
    /// returns the entry list of the given archive file.
    fn list(&self, archive_file: PathBuf) -> Result<Vec<Entry>>;
    /// extract the given archive file into the specified directory with the given options.
    fn perform(&self, archive_file: PathBuf, opts: PathUtils) -> Result<()>;
    #[cfg(test)]
    /// returns the supported format of the extractor.
    fn format(&self) -> Format;
}

/// Returns the extractor for the given archive file.
/// The supported format is `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
fn create<P: AsRef<Path>>(file: P) -> Result<Box<dyn ToteExtractor>> {
    let file = file.as_ref();
    let format = find_format(file);
    match format {
        Ok(format) => match format {
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
        },
        Err(msg) => Err(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination() {
        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts1 = Extractor::builder()
            .archive_file(archive_file)
            .use_archive_name_dir(true)
            .build();
        assert_eq!(opts1.base_dir(), PathBuf::from("./archive"));
        if let Ok(t) = opts1.destination("text1.txt") {
            assert_eq!(t, PathBuf::from("./archive/text1.txt"));
        }
        if let Ok(t) = opts1.destination("text2.txt") {
            assert_eq!(t, PathBuf::from("./archive/text2.txt"));
        }

        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts2 = Extractor::builder().archive_file(archive_file).build();
        assert_eq!(opts2.base_dir(), PathBuf::from("."));
        if let Ok(t) = opts2.destination("./text1.txt") {
            assert_eq!(t, PathBuf::from("./text1.txt"));
        }
    }

    #[test]
    fn test_create_extractor() {
        let e1 = create(&PathBuf::from("results/test.zip"));
        assert!(e1.is_ok());
        assert_eq!(e1.unwrap().format(), Format::Zip);

        let e2 = create(&PathBuf::from("results/test.tar"));
        assert!(e2.is_ok());
        assert_eq!(e2.unwrap().format(), Format::Tar);

        let e3 = create(&PathBuf::from("results/test.tgz"));
        assert!(e3.is_ok());
        assert_eq!(e3.unwrap().format(), Format::TarGz);

        let e4 = create(&PathBuf::from("results/test.tbz2"));
        assert!(e4.is_ok());
        assert_eq!(e4.unwrap().format(), Format::TarBz2);

        let e5 = create(&PathBuf::from("results/test.rar"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::Rar);

        let e6 = create(&PathBuf::from("results/test.tar.xz"));
        assert!(e6.is_ok());
        assert_eq!(e6.unwrap().format(), Format::TarXz);

        let e7 = create(&PathBuf::from("results/test.7z"));
        assert!(e7.is_ok());
        assert_eq!(e7.unwrap().format(), Format::SevenZ);

        let e5 = create(&PathBuf::from("results/test.lzh"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::LHA);

        let e8 = create(&PathBuf::from("results/test.unknown"));
        assert!(e8.is_err());
        match e8 {
            Err(ToteError::UnknownFormat(msg)) => {
                assert_eq!(msg, "test.unknown".to_string());
            }
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(_) => panic!("unexpected success"),
        }
    }
}
