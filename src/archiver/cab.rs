use std::fs::File;
use std::path::PathBuf;

use cab::CabinetBuilder;

use crate::archiver::{Archiver, ArchiverOpts};
use crate::cli::{ToteError, Result};
use crate::format::Format;

pub(super) struct CabArchiver {
}

impl Archiver for CabArchiver {
    fn perform(&self, opts: &ArchiverOpts) -> Result<()> {
        match opts.destination() {
            Err(e) =>  Err(e),
            Ok(file) => {
                write_impl(file, opts.targets(), opts.recursive, opts.base_dir.clone())
            }
        }
    }

    fn format(&self) -> Format {
        Format::Cab
    }
}

fn write_impl(file: File, targets: Vec<PathBuf>, recursive: bool, base_dir: PathBuf) -> Result<()> {
    let mut builder = CabinetBuilder::new();
    let folder = builder.add_folder(cab::CompressionType::MsZip);
    let list = correct_targets(targets, recursive, base_dir);
    for (_from, dest_file) in list.clone() {
        folder.add_file(dest_file);
    }
    let mut writer = match builder.build(file) {
        Ok(w) => w,
        Err(e) => return Err(ToteError::Archiver(e.to_string())),
    };
    let mut iter = list.iter();
    while let Some(mut w) = writer.next_file().unwrap() {
        let (from, _) = iter.next().unwrap();
        if let Ok(mut reader) = File::open(from) {
            std::io::copy(&mut reader, &mut w).unwrap();
        }
    }
    match writer.finish() {
        Ok(_) => Ok(()),
        Err(e) => Err(ToteError::Archiver(e.to_string())),
    }
}

fn correct_targets(targets: Vec<PathBuf>, recursive: bool, base_dir: PathBuf) -> Vec<(PathBuf, String)> {
    let mut result = vec![];
    for target in targets {
        let path = target.as_path();
        if path.is_dir() && recursive {
            process_dir(&mut result, path.to_path_buf(), &base_dir);
        } else {
            process_file(&mut result, path.to_path_buf(), &base_dir);
        }
    }
    result
}

fn process_dir(result: &mut Vec<(PathBuf, String)>, path: PathBuf, base_dir: &PathBuf) {
    for entry in path.read_dir().unwrap() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.is_dir() {
                process_dir(result, e.path(), &base_dir)
            } else if p.is_file() {
                process_file(result, e.path(), &base_dir)
            }
        }
    }
}

fn process_file(result: &mut Vec<(PathBuf, String)>, target: PathBuf, base_dir: &PathBuf) {
    let target_path = match target.strip_prefix(base_dir) {
        Ok(p) => p.to_path_buf(),
        Err(_) => target.clone(),
    };
    let name = target_path.to_str().unwrap();
    result.push((target, name.to_string()));
}