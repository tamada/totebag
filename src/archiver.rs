use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::slice::Iter;

use ignore::{Walk, WalkBuilder};

use crate::format::{find_format, is_archive_file, Format};
use crate::{IgnoreType, Result, ToteError};

mod cab;
mod lha;
mod os;
mod rar;
mod sevenz;
mod tar;
mod zip;

pub trait ToteArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, opts: &ArchiverOpts) -> Result<()>;
    fn format(&self) -> Format;
    fn enable(&self) -> bool;
}

pub struct Archiver<'a> {
    pub dest: PathBuf,
    pub froms: Vec<PathBuf>,
    pub opts: &'a ArchiverOpts,
}

pub struct TargetPath<'a> {
    base_path: &'a PathBuf,
    opts: &'a ArchiverOpts,
}

impl<'a> TargetPath<'a> {
    pub fn new(target: &'a PathBuf, opts: &'a ArchiverOpts) -> Self {
        Self {
            base_path: target,
            opts: opts,
        }
    }

    pub fn dest_path(&self, target: &PathBuf) -> PathBuf {
        // let target_path = if self.base_path.is_dir() {
        //     match target.strip_prefix(&self.base_path) {
        //         Ok(p) => p,
        //         Err(_) => target,
        //     }
        // } else {
        //     if let Some(basedir) = self.base_path.parent() {
        //         match target.strip_prefix(basedir) {
        //             Ok(p) => p,
        //             Err(_) => target,
        //         }
        //     } else {
        //         target
        //     }
        // };
        let target_path = target;
        if let Some(rebase) = &self.opts.rebase_dir {
            rebase.join(target_path)
        } else {
            target_path.to_path_buf()
        }
    }

    pub fn walker(&self) -> Walk {
        let mut builder = WalkBuilder::new(&self.base_path);
        build_walker_impl(self.opts, &mut builder);
        builder.build()
    }
}

impl<'a> Archiver<'a> {
    pub fn create(args: Vec<String>, output: Option<PathBuf>, opts: &'a ArchiverOpts) -> Self {
        let (dest, froms) = find_dest_and_args(args, output);
        Self { dest, froms, opts }
    }

    pub fn new(dest: PathBuf, froms: Vec<PathBuf>, opts: &'a ArchiverOpts) -> Self {
        Self { dest, froms, opts }
    }

    pub fn format(&self) -> Format {
        match create_archiver(&self.dest) {
            Ok(archiver) => archiver.format(),
            Err(e) => Format::Unknown(format!("{:?}", e)),
        }
    }

    pub fn perform(&self) -> Result<()> {
        let archiver = match create_archiver(&self.dest) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };
        if archiver.enable() {
            let paths = self
                .froms
                .iter()
                .map(|item| TargetPath::new(item, &self.opts))
                .collect::<Vec<TargetPath>>();

            match self.destination() {
                Ok(file) => archiver.perform(file, paths, &self.opts),
                Err(e) => Err(e),
            }
        } else {
            Err(ToteError::UnsupportedFormat(self.format().to_string()))
        }
    }

    /// Returns the destination file for the archive with opening it and create the parent directories.
    /// If the path for destination is a directory or exists and overwrite is false,
    /// this function returns an error.
    pub fn destination(&self) -> Result<File> {
        let p = self.dest.as_path();
        log::info!("{:?}: {}", p, p.exists());
        if p.exists() {
            if p.is_dir() {
                return Err(ToteError::DestIsDir(p.to_path_buf()));
            } else if p.is_file() && !self.opts.overwrite {
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
        match File::create(self.dest.as_path()) {
            Ok(f) => Ok(f),
            Err(e) => Err(ToteError::IO(e)),
        }
    }

    pub fn info(&self) -> String {
        format!(
            "Format: {:?}\nDestination: {:?}\nTargets: {:?}",
            self.format(),
            self.dest,
            self.froms
                .iter()
                .map(|item| item.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn build_walker_impl(opts: &ArchiverOpts, w: &mut WalkBuilder) {
    for it in opts.ignore_types() {
        match it {
            IgnoreType::Ignore => w.ignore(true),
            IgnoreType::GitIgnore => w.git_ignore(true),
            IgnoreType::GitGlobal => w.git_global(true),
            IgnoreType::GitExclude => w.git_exclude(true),
            IgnoreType::Default => w
                .ignore(true)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true),
            IgnoreType::Hidden => w.hidden(true),
        };
    }
}

pub fn create_archiver(dest: &PathBuf) -> Result<Box<dyn ToteArchiver>> {
    use crate::archiver::cab::CabArchiver;
    use crate::archiver::lha::LhaArchiver;
    use crate::archiver::rar::RarArchiver;
    use crate::archiver::sevenz::SevenZArchiver;
    use crate::archiver::tar::{
        TarArchiver, TarBz2Archiver, TarGzArchiver, TarXzArchiver, TarZstdArchiver,
    };
    use crate::archiver::zip::ZipArchiver;

    let format = find_format(dest.file_name());
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

pub struct ArchiverOpts {
    pub rebase_dir: Option<PathBuf>,
    pub overwrite: bool,
    pub recursive: bool,
    pub its: Vec<IgnoreType>,
}

fn map_to_path_buf(iter: Iter<String>) -> Vec<PathBuf> {
    iter.map(|item| PathBuf::from(item)).collect()
}

fn find_dest_and_args(args: Vec<String>, output: Option<PathBuf>) -> (PathBuf, Vec<PathBuf>) {
    match output {
        Some(o) => (o.clone(), map_to_path_buf(args.iter())),
        None => {
            let dest = PathBuf::from(&args[0]);
            if is_archive_file(&dest) {
                (dest, map_to_path_buf(args[1..].to_vec().iter()))
            } else {
                (PathBuf::from("totebag.zip"), map_to_path_buf(args.iter()))
            }
        }
    }
}

impl ArchiverOpts {
    pub fn new(
        rebase_dir: Option<PathBuf>,
        overwrite: bool,
        recursive: bool,
        its: Vec<IgnoreType>,
    ) -> Self {
        ArchiverOpts {
            rebase_dir: rebase_dir,
            overwrite: overwrite,
            recursive: recursive,
            its: its,
        }
    }

    #[cfg(test)]
    pub fn create(
        rebase_dir: Option<PathBuf>,
        overwrite: bool,
        recursive: bool,
        its: Vec<IgnoreType>,
    ) -> Self {
        let its = if its.is_empty() {
            vec![IgnoreType::Default]
        } else {
            its
        };
        ArchiverOpts {
            rebase_dir,
            overwrite,
            recursive,
            its: its,
        }
    }

    pub fn ignore_types(&self) -> Vec<IgnoreType> {
        if self.its.is_empty() {
            vec![
                IgnoreType::Ignore,
                IgnoreType::GitIgnore,
                IgnoreType::GitGlobal,
                IgnoreType::GitExclude,
            ]
        } else {
            let mut r = HashSet::<IgnoreType>::new();
            for &it in &self.its {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_path() {
        let opts = ArchiverOpts::create(Some(PathBuf::from("new")), true, true, vec![]);
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &opts);

        assert_eq!(
            PathBuf::from("new/testdata/sample/src/archiver.rs").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/src/archiver.rs"))
        );
    }

    #[test]
    fn test_target_path2() {
        let opts = ArchiverOpts::create(None, true, true, vec![]);
        let base = PathBuf::from("testdata/sample");
        let tp = TargetPath::new(&base, &opts);

        assert_eq!(
            PathBuf::from("testdata/sample/Cargo.toml").as_path(),
            tp.dest_path(&PathBuf::from("testdata/sample/Cargo.toml"))
        );
    }

    #[test]
    fn test_target_path3() {
        let opts = ArchiverOpts::create(Some(PathBuf::from("new")), true, true, vec![]);
        let base = PathBuf::from("testdata/sample/Cargo.toml");
        let tp = TargetPath::new(&base, &opts);

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
        if let Err(e) = ae {
            if let ToteError::UnknownFormat(msg) = e {
                assert_eq!(msg, "test.unknown: unknown format".to_string());
            } else {
                assert!(false);
            }
        }
    }
}
