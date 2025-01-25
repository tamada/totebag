/*!
 * This module provides the extractor for the archive file.
 * The supported formats are `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
 * The extractor is implemented as a trait `ToteExtractor`.
 *
 * # Example: listing the entries in the archive file
 *
 * ```
 * let file = PathBuf::from("testdata/test.zip");
 * match create_extractor(&file) {
 *     Ok(extractor) => match extractor.list() {
 *         Ok(entries) => println!("{:?}", entries),
 *         Err(e) => println!("error: {:?}", e),
 *     },
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 *
 * # Example: extracting the archive file
 *
 * The destination for extraction is the current directory in the following example.
 * [ExtractorOpts] can specify the destination directory and other options.
 *
 * ```
 * let opts = ExtractorOpts::new();
 * match create_extractor(&file) {
 *     Ok(extractor) => match extractor.perform(&opts) {
 *         Ok(r) => println!("{:?}", r),
 *         Err(e) => println!("error: {:?}", e),
 *     },
 *     Err(e) => println!("error: {:?}", e),
 * }
 * ```
 */
use chrono::NaiveDateTime;
use std::fmt::Display;
use std::path::PathBuf;

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

/// The optional parameters for `ToteExtractor`.
pub struct ExtractorOpts {
    pub dest: PathBuf,
    pub use_archive_name_dir: bool,
    pub overwrite: bool,
}

impl ExtractorOpts {
    /// ```
    /// ExtractorOpts::new_with_opts(None, false, false);
    /// ```
    ///
    /// `None` of `dest` means the current directory (`PathBuf::from(".")`).
    pub fn new() -> Self {
        ExtractorOpts::new_with_opts(None, false, false)
    }

    /// create a new ExtractorOpts instance with the given parameters.
    pub fn new_with_opts(
        dest: Option<PathBuf>,
        use_archive_name_dir: bool,
        overwrite: bool,
    ) -> Self {
        ExtractorOpts {
            dest: dest.unwrap_or_else(|| PathBuf::from(".")),
            use_archive_name_dir,
            overwrite,
        }
    }

    /// Returns the base of the destination directory for the archive file.
    /// The target is the archive file name of source.
    pub fn base_dir(&self, archive_file: &PathBuf) -> PathBuf {
        if self.use_archive_name_dir {
            if let Some(stem) = archive_file.file_stem() {
                let dir_name = stem.to_str().unwrap();
                self.dest.join(dir_name)
            } else {
                self.dest.clone()
            }
        } else {
            self.dest.clone()
        }
    }

    /// Return the path of the `target` file for output.
    pub fn destination(&self, archive_file: &PathBuf, target: &PathBuf) -> Result<PathBuf> {
        let base = self.base_dir(archive_file);
        let dest = base.join(target);
        if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest.clone()))
        } else {
            Ok(dest)
        }
    }

    // pub fn format(&self, archive_file: &PathBuf) -> Format {
    //     match format::find_format(archive_file.file_name()) {
    //         Ok(f) => f,
    //         Err(e) => Format::Unknown(format!("{:?}", e).to_string()),
    //     }
    // }

    pub fn can_extract(&self, archive_file: &PathBuf) -> Result<()> {
        let dest = self.base_dir(archive_file);
        if dest == PathBuf::from(".") {
            Ok(())
        } else if dest.exists() && !self.overwrite {
            Err(ToteError::FileExists(dest))
        } else {
            Ok(())
        }
    }
}

// pub struct LegacyExtractor<'a> {
//     opts: &'a ExtractorOpts,
// }

// impl<'a> LegacyExtractor<'a> {
//     pub fn new(opts: &'a ExtractorOpts) -> Self {
//         Self { opts }
//     }

//     pub fn perform(&self, archive_file: &PathBuf) -> Result<()> {
//         let extractor = match create_extractor(archive_file) {
//             Ok(e) => e,
//             Err(e) => return Err(e),
//         };
//         extractor.perform(archive_file, &self.opts)
//     }

//     pub fn list(&self, archive_file: &PathBuf) -> Result<Vec<Entry>> {
//         let extractor = match create_extractor(archive_file) {
//             Ok(e) => e,
//             Err(e) => return Err(e),
//         };
//         extractor.list(archive_file)
//     }

//     pub fn info(&self, archive_file: &PathBuf) -> String {
//         let f = match find_format(archive_file) {
//             Ok(f) => f,
//             Err(e) => Format::Unknown(format!("{:?}", e).to_string()),
//         };
//         format!(
//             "Format: {:?}\nFile: {:?}\nDestination: {:?}",
//             f, archive_file, self.opts.dest,
//         )
//     }
// }

/// The trait for extracting the archive file.
pub trait Extractor {
    /// returns the entry list of the given archive file.
    fn list(&self) -> Result<Vec<Entry>>;
    /// extract the given archive file into the specified directory with the given options.
    fn perform(&self, opts: &ExtractorOpts) -> Result<()>;
    /// returns the supported format of the extractor.
    fn format(&self) -> Format;
}

pub fn info(e: &Box<dyn Extractor>, archive_file: &PathBuf, opts: &ExtractorOpts) -> String {
    format!(
        "Format: {:?}\nFile: {:?}\nDestination: {:?}",
        e.format(),
        archive_file,
        opts.dest,
    )
}

/// Returns the extractor for the given archive file.
/// The supported format is `cab`, `lha`, `rar`, `7z`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
pub fn create(file: PathBuf) -> Result<Box<dyn Extractor>> {
    let format = find_format(&file);
    match format {
        Ok(format) => {
            return match format {
                Format::Cab => Ok(Box::new(cab::CabExtractor::new(file))),
                Format::LHA => Ok(Box::new(lha::LhaExtractor::new(file))),
                Format::Rar => Ok(Box::new(rar::RarExtractor::new(file))),
                Format::SevenZ => Ok(Box::new(sevenz::SevenZExtractor::new(file))),
                Format::Tar => Ok(Box::new(tar::TarExtractor::new(file))),
                Format::TarBz2 => Ok(Box::new(tar::TarBz2Extractor::new(file))),
                Format::TarGz => Ok(Box::new(tar::TarGzExtractor::new(file))),
                Format::TarXz => Ok(Box::new(tar::TarXzExtractor::new(file))),
                Format::TarZstd => Ok(Box::new(tar::TarZstdExtractor::new(file))),
                Format::Zip => Ok(Box::new(zip::ZipExtractor::new(file))),
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
        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts1 = ExtractorOpts::new_with_opts(None, true, false);
        assert_eq!(opts1.base_dir(&archive_file), PathBuf::from("./archive"));
        if let Ok(t) = opts1.destination(&archive_file, &PathBuf::from("text1.txt")) {
            assert_eq!(t, PathBuf::from("./archive/text1.txt"));
        }
        if let Ok(t) = opts1.destination(&archive_file, &PathBuf::from("text2.txt")) {
            assert_eq!(t, PathBuf::from("./archive/text2.txt"));
        }

        let archive_file = PathBuf::from("/tmp/archive.zip");
        let opts2 = ExtractorOpts::new();
        assert_eq!(opts2.base_dir(&archive_file), PathBuf::from("."));
        if let Ok(t) = opts2.destination(&archive_file, &PathBuf::from("./text1.txt")) {
            assert_eq!(t, PathBuf::from("./text1.txt"));
        }
    }

    #[test]
    fn test_create_extractor() {
        let e1 = create(PathBuf::from("results/test.zip"));
        assert!(e1.is_ok());
        assert_eq!(e1.unwrap().format(), Format::Zip);

        let e2 = create(PathBuf::from("results/test.tar"));
        assert!(e2.is_ok());
        assert_eq!(e2.unwrap().format(), Format::Tar);

        let e3 = create(PathBuf::from("results/test.tgz"));
        assert!(e3.is_ok());
        assert_eq!(e3.unwrap().format(), Format::TarGz);

        let e4 = create(PathBuf::from("results/test.tbz2"));
        assert!(e4.is_ok());
        assert_eq!(e4.unwrap().format(), Format::TarBz2);

        let e5 = create(PathBuf::from("results/test.rar"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::Rar);

        let e6 = create(PathBuf::from("results/test.tar.xz"));
        assert!(e6.is_ok());
        assert_eq!(e6.unwrap().format(), Format::TarXz);

        let e7 = create(PathBuf::from("results/test.7z"));
        assert!(e7.is_ok());
        assert_eq!(e7.unwrap().format(), Format::SevenZ);

        let e5 = create(PathBuf::from("results/test.lzh"));
        assert!(e5.is_ok());
        assert_eq!(e5.unwrap().format(), Format::LHA);

        let e8 = create(PathBuf::from("results/test.unknown"));
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
