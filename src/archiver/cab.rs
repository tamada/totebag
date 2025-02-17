use std::fs::File;
use std::path::PathBuf;

use cab::{CabinetBuilder, CabinetWriter};

use crate::archiver::{ArchiveEntry, TargetPath, Targets, ToteArchiver};
use crate::{Result, ToteError};

pub(super) struct CabArchiver {}

impl ToteArchiver for CabArchiver {
    fn perform(&self, file: File, tps: Targets) -> Result<Vec<ArchiveEntry>> {
        let mut errs = vec![];
        let mut entries = vec![];
        let mut builder = CabinetBuilder::new();
        let ctype = compression_type(tps.level());
        let folder = builder.add_folder(ctype);
        let list = collect_entries(&tps);
        for (path, tp) in list.clone() {
            entries.push(ArchiveEntry::from(&path));
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
            Ok(_) => Ok(entries),
            Err(e) => Err(ToteError::Archiver(e.to_string())),
        }
    }

    fn enable(&self) -> bool {
        true
    }
}

fn compression_type(level: u8) -> cab::CompressionType {
    match level {
        0 => cab::CompressionType::None,
        _ => cab::CompressionType::MsZip,
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

fn collect_entries<'a>(tps: &'a Targets) -> Vec<(PathBuf, &'a TargetPath<'a>)> {
    let mut r = vec![];
    for tp in tps.iter() {
        for t in tp.iter() {
            let path = t.into_path();
            if path.is_file() {
                r.push((path, tp));
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
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
    fn test_archive() {
        run_test(|| {
            let archiver = Archiver::builder()
                .archive_file(PathBuf::from("results/test.cab"))
                .targets(vec![PathBuf::from("src"), PathBuf::from("Cargo.toml")])
                .overwrite(false)
                .no_recursive(true)
                .build();
            let r = archiver.perform();
            assert!(r.is_ok());
        });
    }

    fn teardown() {
        let _ = std::fs::remove_file("results/test.cab");
    }
}
