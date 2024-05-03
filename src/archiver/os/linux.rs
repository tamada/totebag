use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use time::OffsetDateTime;
use zip::write::SimpleFileOptions;
use zip::DateTime;

pub fn create(target: &PathBuf) -> SimpleFileOptions {
    let metadata = std::fs::metadata(&target).unwrap();
    let mod_time = DateTime::try_from(
        OffsetDateTime::from(metadata.modified().unwrap()));

    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(metadata.permissions().mode())
}