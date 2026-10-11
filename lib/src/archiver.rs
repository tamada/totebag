//! This module provides an interface and struct for archiving the files.
//! The supported formats are: `ar`, `cab`, `cpio`, `7z`, `tar`, `tar.gz`,
//! `tar.bz2`, `tar.xz`, `tar.zst`, and `zip`.
//! `lha` and `rar` can only be extracted, never created.
//!
//! # Example: archiving the files
//!
//! ```
//! use std::path::PathBuf;
//!
//! let config = totebag::ArchiveConfig::builder()
//!     .dest("results/test.zip")                  // destination file.
//!     .rebase_dir(PathBuf::from("new"))          // rebased directory in the archive file.
//!     .overwrite(true)                           // set overwrite flag of the destination file.
//!     .build();
//! let targets = vec!["src", "Cargo.toml"].iter() // files to be archived.
//!     .map(|s| PathBuf::from(s)).collect::<Vec<PathBuf>>();
//! match totebag::archive(&targets, &config) {
//!     Ok(_) => println!("archiving is done"),
//!     Err(e) => eprintln!("error: {:?}", e),
//! }
//! ```
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crate::format::default_format_detector;
use crate::{Error, Result};

mod ar;
mod cab;
mod cpio;
mod lha;
mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

/// Represents a set of entries for archiving into the resultant file.
#[derive(Debug)]
pub struct ArchiveEntries {
    pub archive_file: PathBuf,
    pub entries: Vec<ArchiveEntry>,
    /// resultant file size (compressed size).
    pub compressed: u64,
}

/// Represents each entry in the archive file.
#[derive(Debug)]
pub struct ArchiveEntry {
    /// the source path of the entry.
    pub path: PathBuf,
    /// original file size.
    pub size: u64,
}

impl ArchiveEntries {
    pub fn new<P: AsRef<Path>>(path: P, entries: Vec<ArchiveEntry>, compressed: u64) -> Self {
        Self {
            archive_file: path.as_ref().to_path_buf(),
            entries,
            compressed,
        }
    }

    /// Returns the total file size of the entries.
    pub fn total(&self) -> u64 {
        self.entries.iter().map(|e| e.size).sum()
    }

    /// the length of the entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the entries is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ArchiveEntry {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size }
    }

    /// Create an instance of ArchiveEntry from the given path by obtaining the file size.
    pub fn from<P: AsRef<Path>>(p: P) -> Self {
        let path = p.as_ref().to_path_buf();
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        Self::new(path, size)
    }
}

/// The trait for creating an archive file.
/// If you want to support archiving for a new format, you need to implement the `ToteArchiver` trait.
/// Then, the call [`perform`](ToteArchiver::perform) method of [`ToteArchiver`].
pub trait ToteArchiver {
    /// Perform the archiving operation.
    /// - `writer` is a buffered writer for the destination file. The caller
    ///   flushes it after this returns, so implementations must not rely on
    ///   dropping it to report write errors; finish any encoder they wrap it in.
    /// - `targets` is the list of files to be archived.
    fn perform(
        &self,
        writer: &mut BufWriter<File>,
        targets: &[Target],
        config: &crate::ArchiveConfig,
    ) -> Result<Vec<ArchiveEntry>>;

    /// Returns true if this archiver is enabled.
    fn enable(&self) -> bool;
}

/// A file or directory to archive: where it is on disk, and its name in the
/// archive.
pub struct Target {
    /// The file or directory on disk, as found by walking the targets.
    pub path: PathBuf,
    /// Its name in the archive, from
    /// [`ArchiveConfig::path_in_archive`](crate::ArchiveConfig::path_in_archive);
    /// never empty.
    pub dest_path: PathBuf,
}

/// Walks `targets` with the ignore settings of `config`, and pairs every file
/// and directory found with its name in the archive.
///
/// A path whose name is empty -- `base_dir` itself, or a directory such as
/// `..` -- has no name to store it under, and its contents are archived
/// anyway, so it is skipped.
///
/// # Errors
///
/// Every problem is collected before returning, so that all of them are
/// reported at once:
///
/// * [`Error::FileNotFound`] for a target that does not exist. The walk would
///   otherwise yield nothing for it, and the archive would silently lack it.
/// * the errors of
///   [`ArchiveConfig::path_in_archive`](crate::ArchiveConfig::path_in_archive),
///   such as a path outside `base_dir`.
///
/// `totebag::archive` calls this before it creates the destination, so an
/// error leaves the destination untouched.
pub(crate) fn walk_targets(
    targets: &[PathBuf],
    config: &crate::ArchiveConfig,
) -> Result<Vec<Target>> {
    let mut r = vec![];
    let mut errs = vec![];
    for tp in targets {
        if !tp.exists() {
            errs.push(Error::FileNotFound(tp.clone()));
            continue;
        }
        for entry in config.iter(tp) {
            let path = entry.into_path();
            match config.path_in_archive(&path) {
                Ok(dest_path) if !dest_path.as_os_str().is_empty() => {
                    r.push(Target { path, dest_path })
                }
                Ok(_) => {}
                Err(e) => errs.push(e),
            }
        }
    }
    Error::error_or(r, errs)
}

pub fn create<P: AsRef<Path>>(dest: P) -> Result<Box<dyn ToteArchiver>> {
    let dest = dest.as_ref();
    let fd = default_format_detector();
    let format = fd.detect(dest);
    match format {
        Some(format) => {
            let archiver: Box<dyn ToteArchiver> = match format.name.as_str() {
                "Ar" => Box::new(ar::Archiver {}),
                "Cab" => Box::new(cab::Archiver {}),
                "Cpio" => Box::new(cpio::Archiver {}),
                "Lha" => Box::new(lha::Archiver {}),
                "Rar" => Box::new(rar::Archiver {}),
                "SevenZ" => Box::new(sevenz::Archiver {}),
                "Tar" => Box::new(tar::Archiver {}),
                "TarBz2" => Box::new(tar::Bz2Archiver {}),
                "TarGz" => Box::new(tar::GzArchiver {}),
                "TarXz" => Box::new(tar::XzArchiver {}),
                "TarZstd" => Box::new(tar::ZstdArchiver {}),
                "Zip" => Box::new(zip::Archiver {}),
                _ => return Err(Error::UnknownFormat(format.name.clone())),
            };
            if archiver.enable() {
                Ok(archiver)
            } else {
                Err(Error::unsupported_for_archiving(&format.name))
            }
        }
        None => Err(Error::Archiver(format!(
            "{}: no suitable archiver",
            dest.display()
        ))),
    }
}

/// Helpers shared by the per-format archiver tests.
#[cfg(test)]
pub(crate) mod test_support {
    use std::path::PathBuf;

    /// The targets every archiver test packs.
    ///
    /// Unit tests run with the working directory set to `lib/`, so this reaches
    /// out of the crate. The previous `["lib", "cli", "Cargo.toml"]` list
    /// resolved to nothing from `lib/` and archived almost nothing (issue #89).
    ///
    /// The leading `..` is deliberate: it also covers the entry-name
    /// normalization that keeps `..` out of the archive (issue #90).
    pub(crate) fn targets() -> Vec<PathBuf> {
        vec![PathBuf::from("../testdata/sample")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archiver() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("rebasedir")
            .overwrite(true)
            .build();
        if let Ok(p) = config.dest_file() {
            assert_eq!(PathBuf::from("results/test.zip"), p);
        }
        assert!(config.overwrite);
        assert!(!config.no_recursive);
        assert_eq!(1, config.ignore.len());
        assert!(config.dest_file().is_ok())
    }

    #[test]
    fn test_target_path() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("new")
            .overwrite(true)
            .build();

        assert_eq!(
            PathBuf::from("new/testdata/sample/src/archiver.rs").as_path(),
            config
                .path_in_archive("testdata/sample/src/archiver.rs")
                .unwrap()
        );
    }

    #[test]
    fn test_target_path2() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            //            .rebase_dir(None)
            .overwrite(true)
            .build();

        assert_eq!(
            PathBuf::from("testdata/sample/Cargo.toml").as_path(),
            config
                .path_in_archive("testdata/sample/Cargo.toml")
                .unwrap()
        );
    }

    /// Entry names must never climb out of the archive or start at the
    /// filesystem root (issue #90).
    #[test]
    fn test_path_in_archive_is_normalized() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .build();
        for (given, expected) in [
            (
                "../testdata/sample/Cargo.toml",
                "testdata/sample/Cargo.toml",
            ),
            ("../../testdata/Cargo.toml", "testdata/Cargo.toml"),
            ("./src/lib.rs", "src/lib.rs"),
            ("src/../lib.rs", "lib.rs"),
            ("/etc/hosts", "etc/hosts"),
            ("src/lib.rs", "src/lib.rs"),
        ] {
            assert_eq!(
                config.path_in_archive(given).unwrap(),
                PathBuf::from(expected),
                "for {given}"
            );
        }
    }

    /// The rebase prefix is applied on top of the normalized path, and is itself
    /// normalized.
    #[test]
    fn test_path_in_archive_rebase_is_normalized() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("../root")
            .build();
        assert_eq!(
            config.path_in_archive("../testdata/Cargo.toml").unwrap(),
            PathBuf::from("root/testdata/Cargo.toml")
        );
    }

    /// Without an explicit rebase directory, entry names keep their own paths --
    /// no `./` prefix.
    #[test]
    fn test_path_in_archive_without_rebase() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .build();
        assert_eq!(config.rebase_dir, None);
        assert_eq!(
            config
                .path_in_archive("testdata/sample/Cargo.toml")
                .unwrap(),
            PathBuf::from("testdata/sample/Cargo.toml")
        );
    }

    #[test]
    fn test_target_path3() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .rebase_dir("new")
            .overwrite(true)
            .build();
        assert_eq!(
            PathBuf::from("new/testdata/sample/Cargo.toml").as_path(),
            config
                .path_in_archive("testdata/sample/Cargo.toml")
                .unwrap()
        );
    }

    /// `base_dir` is stripped from the front of the entry name (issue #108).
    #[test]
    fn test_path_in_archive_strips_base_dir() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .base_dir("testdata/sample")
            .build();
        for (given, expected) in [
            ("testdata/sample/Cargo.toml", "Cargo.toml"),
            ("testdata/sample/src/lib.rs", "src/lib.rs"),
            ("./testdata/sample/src/lib.rs", "src/lib.rs"),
            ("testdata/other/../sample/Cargo.toml", "Cargo.toml"),
            // base_dir itself has no name; the archivers skip it
            ("testdata/sample", ""),
        ] {
            assert_eq!(
                config.path_in_archive(given).unwrap(),
                PathBuf::from(expected),
                "for {given}"
            );
        }
    }

    /// Paths are compared as absolute paths, so a relative base_dir matches an
    /// absolute target and the other way round.
    #[test]
    fn test_path_in_archive_base_dir_relative_and_absolute() {
        let cwd = std::env::current_dir().unwrap();
        let absolute_base = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .base_dir(cwd.join("testdata"))
            .build();
        assert_eq!(
            absolute_base
                .path_in_archive("testdata/sample/Cargo.toml")
                .unwrap(),
            PathBuf::from("sample/Cargo.toml")
        );
        let relative_base = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .base_dir("testdata")
            .build();
        assert_eq!(
            relative_base
                .path_in_archive(cwd.join("testdata/sample/Cargo.toml"))
                .unwrap(),
            PathBuf::from("sample/Cargo.toml")
        );
    }

    /// A path outside base_dir is an error, including one whose *normalized*
    /// form would look as if it were inside: `../x` is not `x`.
    #[test]
    fn test_path_in_archive_outside_base_dir() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .base_dir("../x")
            .build();
        for given in ["x/a.txt", "src/lib.rs", "../y/a.txt"] {
            assert!(
                matches!(
                    config.path_in_archive(given),
                    Err(crate::Error::OutsideBaseDir(_))
                ),
                "{given} must be rejected"
            );
        }
        assert_eq!(
            config.path_in_archive("../x/a.txt").unwrap(),
            PathBuf::from("a.txt")
        );
    }

    /// base_dir is stripped first, then rebase_dir is prefixed.
    #[test]
    fn test_path_in_archive_base_dir_and_rebase_dir() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/test.zip")
            .base_dir("testdata/sample")
            .rebase_dir("root")
            .build();
        assert_eq!(
            config
                .path_in_archive("testdata/sample/Cargo.toml")
                .unwrap(),
            PathBuf::from("root/Cargo.toml")
        );
    }

    /// End to end: archiving testdata/sample with base_dir set to it stores the
    /// files at the top level of the archive.
    #[test]
    fn test_archive_with_base_dir() {
        let dest = PathBuf::from("results/base_dir.tar");
        let config = crate::ArchiveConfig::builder()
            .dest(&dest)
            .base_dir("../testdata/sample")
            .overwrite(true)
            .build();
        crate::archive(&test_support::targets(), &config).unwrap();
        let list_config = crate::ListConfig::new(
            crate::OutputFormat::Default,
            crate::format::default_format_detector(),
        );
        let names = crate::list(&dest, &list_config)
            .map(|s| s.lines().map(str::to_string).collect::<Vec<_>>());
        let _ = std::fs::remove_file(&dest);
        let names = names.unwrap();
        assert!(names.contains(&"Cargo.toml".to_string()), "{names:?}");
        assert!(
            names.iter().all(|n| !n.starts_with("testdata")),
            "{names:?}"
        );
    }

    /// A target outside base_dir is an error found before the destination is
    /// created: an existing archive keeps its contents even with `overwrite`,
    /// and no new file is left behind.
    #[test]
    fn test_archive_outside_base_dir_leaves_destination_untouched() {
        std::fs::create_dir_all("results").unwrap();
        let targets = vec![PathBuf::from("../testdata/sample"), PathBuf::from("src")];

        let existing = PathBuf::from("results/outside_base_existing.zip");
        std::fs::write(&existing, b"precious").unwrap();
        let config = crate::ArchiveConfig::builder()
            .dest(&existing)
            .base_dir("../testdata/sample")
            .overwrite(true)
            .build();
        let r = crate::archive(&targets, &config);
        let kept = std::fs::read(&existing).unwrap();
        let _ = std::fs::remove_file(&existing);
        match r {
            Err(crate::Error::Array(errs)) => assert!(
                errs.iter()
                    .all(|e| matches!(e, crate::Error::OutsideBaseDir(_))),
                "{errs:?}"
            ),
            Err(crate::Error::OutsideBaseDir(_)) => {}
            other => panic!("unexpected result: {other:?}"),
        }
        assert_eq!(kept, b"precious");

        let fresh = PathBuf::from("results/outside_base_fresh.zip");
        let _ = std::fs::remove_file(&fresh);
        let config = crate::ArchiveConfig::builder()
            .dest(&fresh)
            .base_dir("../testdata/sample")
            .build();
        assert!(crate::archive(&targets, &config).is_err());
        assert!(!fresh.exists());
    }

    /// A target that does not exist is reported, and nothing is written
    /// (issue #52). It used to be skipped silently, producing an empty or
    /// incomplete archive and a successful exit.
    #[test]
    fn test_archive_missing_target() {
        std::fs::create_dir_all("results").unwrap();
        for (targets, case) in [
            (vec![PathBuf::from("no/such/dir")], "only a missing target"),
            (
                vec![
                    PathBuf::from("../testdata/sample"),
                    PathBuf::from("no/such/file.txt"),
                ],
                "an existing and a missing target",
            ),
        ] {
            let dest = PathBuf::from("results/missing_target.zip");
            let _ = std::fs::remove_file(&dest);
            let config = crate::ArchiveConfig::builder().dest(&dest).build();
            match crate::archive(&targets, &config) {
                Err(crate::Error::FileNotFound(p)) => {
                    assert_eq!(p, *targets.last().unwrap(), "{case}")
                }
                other => panic!("{case}: unexpected result: {other:?}"),
            }
            assert!(!dest.exists(), "{case}: no archive must be created");
        }
    }

    /// Every missing target is reported, not only the first one.
    #[test]
    fn test_archive_reports_every_missing_target() {
        let config = crate::ArchiveConfig::builder()
            .dest("results/missing_targets.zip")
            .build();
        let targets = vec![PathBuf::from("no/such/a"), PathBuf::from("no/such/b")];
        match crate::archive(&targets, &config) {
            Err(crate::Error::Array(errs)) => {
                assert_eq!(errs.len(), 2);
                assert!(
                    errs.iter()
                        .all(|e| matches!(e, crate::Error::FileNotFound(_)))
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
        assert!(!PathBuf::from("results/missing_targets.zip").exists());
    }
}
