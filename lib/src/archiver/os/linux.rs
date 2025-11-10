use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use time::OffsetDateTime;
use zip::write::SimpleFileOptions;
use zip::DateTime;

pub(super) fn create_file_opts(target: &PathBuf, level: i64) -> SimpleFileOptions {
    let metadata = std::fs::metadata(target).unwrap();
    let mod_time = DateTime::try_from(OffsetDateTime::from(metadata.modified().unwrap()));
    let (method, level) = super::method_and_level(level);

    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(method)
        .compression_level(level)
        .unix_permissions(metadata.permissions().mode())
}
