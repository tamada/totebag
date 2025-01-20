use std::fs::File;
use std::path::PathBuf;

use cab::{CabinetBuilder, CabinetWriter};

use crate::archiver::{ArchiverOpts, TargetPath, ToteArchiver};
use crate::format::Format;
use crate::{Result, ToteError};

pub(super) struct CabArchiver {}

impl ToteArchiver for CabArchiver {
    fn perform(&self, file: File, tps: Vec<TargetPath>, _opts: &ArchiverOpts) -> Result<()> {
        let mut errs = vec![];
        let mut builder = CabinetBuilder::new();
        let folder = builder.add_folder(cab::CompressionType::MsZip);
        let list = collect_entries(&tps);
        for (path, tp) in list.clone() {
            folder.add_file(tp.dest_path(&path).to_str().unwrap().to_string());
        }
        let mut writer = match builder.build(file) {
            Ok(w) => w,
            Err(e) => return Err(ToteError::Archiver(e.to_string())),
        };
        for (path, _) in list {
            if let Err(e) = write_entry(&mut writer, path) {
                errs.push(e);
            }
        }
        match writer.finish() {
            Ok(_) => Ok(()),
            Err(e) => Err(ToteError::Archiver(e.to_string())),
        }
    }

    fn format(&self) -> Format {
        Format::Cab
    }

    fn enable(&self) -> bool {
        true
    }
}

fn write_entry(writer: &mut CabinetWriter<File>, path: PathBuf) -> Result<()> {
    match (File::open(path), writer.next_file()) {
        (Ok(mut reader), Ok(Some(mut w))) => match std::io::copy(&mut reader, &mut w) {
            Ok(_) => Ok(()),
            Err(e) => Err(ToteError::IO(e)),
        },
        (_, Ok(None)) => Err(ToteError::Archiver("cab writer error".to_string())),
        (Err(e1), Err(e2)) => Err(ToteError::Array(vec![
            ToteError::IO(e1),
            ToteError::Fatal(Box::new(e2)),
        ])),
        (Err(e), _) => Err(ToteError::IO(e)),
        (_, Err(e)) => Err(ToteError::Archiver(e.to_string())),
    }
}

fn collect_entries<'a>(tps: &'a Vec<TargetPath>) -> Vec<(PathBuf, &'a TargetPath<'a>)> {
    let mut r = vec![];
    for tp in tps {
        for entry in tp.walker() {
            if let Ok(t) = entry {
                let path = t.into_path();
                if path.is_file() {
                    r.push((path, tp));
                }
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::archiver::Archiver;
    use std::path::PathBuf;

    fn run_test<F>(f: F)
    where
        F: FnOnce(),
    {
        // setup(); // 予めやりたい処理
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        teardown(); // 後片付け処理

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_format() {
        let archiver = CabArchiver {};
        assert_eq!(archiver.format(), Format::Cab);
    }

    #[test]
    fn test_archive() {
        run_test(|| {
            let opts = ArchiverOpts {
                rebase_dir: None,
                overwrite: false,
                recursive: false,
                its: vec![],
            };
            let archiver = Archiver::new(
                PathBuf::from("results/test.cab"),
                vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")],
                &opts,
            );
            let r = archiver.perform();
            assert!(r.is_ok());
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.cab");
    }
}
