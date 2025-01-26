use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use ignore::{Walk, WalkBuilder};
use typed_builder::TypedBuilder;

use crate::format::{find_format, Format};
use crate::{IgnoreType, Result, ToteError};

mod cab;
mod lha;
mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub(crate) trait ToteArchiver {
    /// Perform the archiving operation.
    /// - `file` is the destination file for the archive.
    /// - `tps` is the list of files to be archived.
    fn perform(&self, file: File, tps: Vec<TargetPath>) -> Result<()>;

    /// Returns the format object of this archiver.
    fn format(&self) -> Format;
    /// Returns true if this archiver is enabled.
    fn enable(&self) -> bool;
}

/// Archiver is a struct to handle the archiving operation.
/// ```
/// let archiver = Archiver::builder()
///     .archive_file(PathBuf::from("results/test.zip"))
///     .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
///     .overwrite(true)      // default is false
/// //    .no_recursive(true)  // default is false
/// //    .ignore_types(vec![IgnoreType::Ignore])  // default is [IgnoreType::Default]
///    .build();
/// match archiver.perform() {
///    Ok(_)  => println!("archiving is done"),
///    Err(e) => eprintln!("error: {:?}", e),
/// }
/// ```
#[derive(Debug, TypedBuilder)]
pub struct Archiver {
    #[builder(setter(into))]
    pub archive_file: PathBuf,
    pub targets: Vec<PathBuf>,
    #[builder(default = None, setter(strip_option, into))]
    pub rebase_dir: Option<PathBuf>,
    #[builder(default = false)]
    pub overwrite: bool,
    #[builder(default = false)]
    pub no_recursive: bool,
    #[builder(default = vec![IgnoreType::Default])]
    pub ignore_types: Vec<IgnoreType>,
}

/// TargetPath is a helper struct to handle the target path for the archiving operation.
pub(crate) struct TargetPath<'a> {
    base_path: &'a PathBuf,
    opts: &'a Archiver,
}

impl<'a> TargetPath<'a> {
    pub fn new(target: &'a PathBuf, archiver: &'a Archiver) -> Self {
        Self {
            base_path: target,
            opts: archiver,
        }
    }

    /// Returns the destination path for the target file.
    pub fn dest_path(&self, target: &PathBuf) -> PathBuf {
        let target_path = target;
        if let Some(rebase) = &self.opts.rebase_dir {
            rebase.join(target_path)
        } else {
            target_path.to_path_buf()
        }
    }

    /// Returns the directory traversing walker for the given path of this instance.
    pub fn walker(&self) -> Walk {
        let mut builder = WalkBuilder::new(&self.base_path);
        build_walker_impl(self.opts, &mut builder);
        builder.build()
    }
}

impl<'a> Archiver {
    pub fn perform(&self) -> Result<()> {
        let archiver = match create_archiver(&self.archive_file) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };
        if !archiver.enable() {
            return Err(ToteError::UnsupportedFormat(format!(
                "{:?}: not support archiving",
                archiver.format()
            )));
        }
        let paths = self
            .targets
            .iter()
            .map(|item| TargetPath::new(item, &self))
            .collect::<Vec<TargetPath>>();

        log::info!("{:?}: {}", self.archive_file, self.archive_file.exists());
        if self.archive_file.exists() {
            if self.archive_file.is_dir() {
                return Err(ToteError::DestIsDir(self.archive_file.clone()));
            } else if self.archive_file.is_file() && !self.overwrite {
                return Err(ToteError::FileExists(self.archive_file.clone()));
            }
        }
        if let Some(parent) = self.archive_file.parent() {
            if !parent.exists() {
                if let Err(e) = create_dir_all(parent) {
                    return Err(ToteError::IO(e));
                }
            }
        }
        match File::create(&self.archive_file) {
            Ok(f) => archiver.perform(f, paths),
            Err(e) => Err(ToteError::IO(e)),
        }
    }

    /// Returns the destination file for the archive with opening it and create the parent directories.
    /// If the path for destination is a directory or exists and overwrite is false,
    /// this function returns an error.
    pub fn destination(&self) -> Result<File> {
        let p = self.archive_file.as_path();
        log::info!("{:?}: {}", p, p.exists());
        if p.exists() {
            if p.is_dir() {
                return Err(ToteError::DestIsDir(p.to_path_buf()));
            } else if p.is_file() && !self.overwrite {
                return Err(ToteError::FileExists(p.to_path_buf()));
            }
        }
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                if let Err(e) = create_dir_all(parent) {
                    return Err(ToteError::IO(e));
                }
            }
        }
        match File::create(&self.archive_file) {
            Ok(f) => Ok(f),
            Err(e) => Err(ToteError::IO(e)),
        }
    }

    pub fn format(&self) -> Format {
        find_format(&self.archive_file).unwrap()
    }

    pub fn info(&self) -> String {
        format!(
            "Format: {:?}\nArchive File: {:?}\nTargets: {:?}",
            find_format(&self.archive_file),
            self.archive_file,
            self.targets
                .iter()
                .map(|item| item.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn ignore_types(&self) -> Vec<IgnoreType> {
        if self.ignore_types.is_empty() {
            vec![
                IgnoreType::Ignore,
                IgnoreType::GitIgnore,
                IgnoreType::GitGlobal,
                IgnoreType::GitExclude,
            ]
        } else {
            let mut r = HashSet::<IgnoreType>::new();
            for &it in &self.ignore_types {
                if it == IgnoreType::Default {
                    r.insert(IgnoreType::Ignore);
                    r.insert(IgnoreType::GitIgnore);
                    r.insert(IgnoreType::GitGlobal);
                    r.insert(IgnoreType::GitExclude);
                } else {
                    r.insert(it.clone());
                }
            }
            r.into_iter().collect()
        }
    }
}

fn build_walker_impl(opts: &Archiver, w: &mut WalkBuilder) {
    for it in opts.ignore_types() {
        match it {
            IgnoreType::Default => w
                .ignore(true)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true),
            IgnoreType::GitIgnore => w.git_ignore(true),
            IgnoreType::GitGlobal => w.git_global(true),
            IgnoreType::GitExclude => w.git_exclude(true),
            IgnoreType::Hidden => w.hidden(true),
            IgnoreType::Ignore => w.ignore(true),
        };
    }
}

fn create_archiver(dest: &PathBuf) -> Result<Box<dyn ToteArchiver>> {
    use crate::archiver::cab::CabArchiver;
    use crate::archiver::lha::LhaArchiver;
    use crate::archiver::rar::RarArchiver;
    use crate::archiver::sevenz::SevenZArchiver;
    use crate::archiver::tar::{
        TarArchiver, TarBz2Archiver, TarGzArchiver, TarXzArchiver, TarZstdArchiver,
    };
    use crate::archiver::zip::ZipArchiver;

    let format = find_format(dest);
    match format {
        Ok(format) => {
            return match format {
                Format::Cab => Ok(Box::new(CabArchiver {})),
                Format::LHA => Ok(Box::new(LhaArchiver {})),
                Format::Rar => Ok(Box::new(RarArchiver {})),
                Format::SevenZ => Ok(Box::new(SevenZArchiver {})),
                Format::Tar => Ok(Box::new(TarArchiver {})),
                Format::TarBz2 => Ok(Box::new(TarBz2Archiver {})),
                Format::TarGz => Ok(Box::new(TarGzArchiver {})),
                Format::TarXz => Ok(Box::new(TarXzArchiver {})),
                Format::TarZstd => Ok(Box::new(TarZstdArchiver {})),
                Format::Zip => Ok(Box::new(ZipArchiver::new())),
                _ => Err(ToteError::UnknownFormat(format.to_string())),
            }
        }
        Err(msg) => Err(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_path() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            .rebase_dir(PathBuf::from("new"))
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("new/testdata/sample/src/archiver.rs").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/src/archiver.rs"))
        );
    }

    #[test]
    fn test_target_path2() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            //            .rebase_dir(None)
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("testdata/sample/Cargo.toml").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/Cargo.toml"))
        );
    }

    #[test]
    fn test_target_path3() {
        let archiver = Archiver::builder()
            .archive_file(PathBuf::from("results/test.zip"))
            .rebase_dir(PathBuf::from("new"))
            .overwrite(true)
            .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
            .build();
        let base = PathBuf::from("testdata/sample/Cargo.toml");
        let tp = TargetPath::new(&base, &archiver);

        assert_eq!(
            PathBuf::from("new/testdata/sample/Cargo.toml").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/Cargo.toml"))
        );
    }

    #[test]
    fn test_archiver() {
        let a1 = create_archiver(&PathBuf::from("results/test.tar"));
        if let Ok(f) = a1 {
            assert_eq!(f.format(), Format::Tar);
        } else {
            assert!(false);
        }

        let a2 = create_archiver(&PathBuf::from("results/test.tar.gz"));
        assert!(a2.is_ok());
        assert_eq!(a2.unwrap().format(), Format::TarGz);

        let a3 = create_archiver(&PathBuf::from("results/test.tar.bz2"));
        assert!(a3.is_ok());
        assert_eq!(a3.unwrap().format(), Format::TarBz2);

        let a4 = create_archiver(&PathBuf::from("results/test.zip"));
        assert!(a4.is_ok());
        assert_eq!(a4.unwrap().format(), Format::Zip);

        let a5 = create_archiver(&PathBuf::from("results/test.rar"));
        assert!(a5.is_ok());
        assert_eq!(a5.unwrap().format(), Format::Rar);

        let a6 = create_archiver(&PathBuf::from("results/test.tar.xz"));
        assert!(a6.is_ok());
        assert_eq!(a6.unwrap().format(), Format::TarXz);

        let a7 = create_archiver(&PathBuf::from("results/test.7z"));
        assert!(a7.is_ok());
        assert_eq!(a7.unwrap().format(), Format::SevenZ);

        let a8 = create_archiver(&PathBuf::from("results/test.tar.zst"));
        assert!(a8.is_ok());
        assert_eq!(a8.unwrap().format(), Format::TarZstd);

        let a9 = create_archiver(&PathBuf::from("results/test.lha"));
        assert!(a9.is_ok());
        assert_eq!(a9.unwrap().format(), Format::LHA);

        let a10 = create_archiver(&PathBuf::from("results/test.cab"));
        assert!(a10.is_ok());
        assert_eq!(a10.unwrap().format(), Format::Cab);

        let ae = create_archiver(&PathBuf::from("results/test.unknown"));
        assert!(ae.is_err());
        match ae {
            Err(ToteError::UnknownFormat(msg)) => assert_eq!(msg, "test.unknown".to_string()),
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(_) => panic!("unexpected result"),
        }
    }
}
