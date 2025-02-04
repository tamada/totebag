use time::OffsetDateTime;
use zip::write::SimpleFileOptions;
use zip::DateTime;

use std::path::PathBuf;

pub(super) fn create_file_opts(target: &PathBuf) -> SimpleFileOptions {
    let metadata = std::fs::metadata(target).unwrap();
    let mod_time = DateTime::try_from(OffsetDateTime::from(metadata.modified().unwrap()));

    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(zip::CompressionMethod::Stored)
}
