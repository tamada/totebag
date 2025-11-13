use std::fs::Metadata;
use std::path::Path;

use time::OffsetDateTime;
use zip::DateTime;
use zip::write::SimpleFileOptions;

pub(super) fn create_file_opts(target: &Path, level: i64) -> SimpleFileOptions {
    let metadata = std::fs::metadata(target).unwrap();
    create_file_option(metadata, level)
}

#[cfg(not(target_os = "windows"))]
fn create_file_option(metadata: Metadata, level: i64) -> SimpleFileOptions {
    use std::os::unix::fs::PermissionsExt;
    let mod_time = DateTime::try_from(OffsetDateTime::from(metadata.modified().unwrap()));
    let (method, level) = method_and_level(level);
    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(method)
        .compression_level(level)
        .unix_permissions(metadata.permissions().mode())
}

#[cfg(target_os = "windows")]
fn create_file_option(metadata: Metadata, level: i64) -> SimpleFileOptions {
    let mod_time = DateTime::try_from(OffsetDateTime::from(metadata.modified().unwrap()));
    let (method, level) = method_and_level(level);
    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(method)
        .compression_level(level)
}

pub(crate) fn method_and_level(level: i64) -> (zip::CompressionMethod, Option<i64>) {
    match level {
        0 => (zip::CompressionMethod::Stored, None),
        1 => (zip::CompressionMethod::Deflated, Some(10)),
        2 => (zip::CompressionMethod::Deflated, Some(24)),
        3 => (zip::CompressionMethod::Deflated, Some(264)),
        4 => (zip::CompressionMethod::Bzip2, Some(1)),
        5 => (zip::CompressionMethod::Bzip2, Some(6)),
        6 => (zip::CompressionMethod::Bzip2, Some(9)),
        7 => (zip::CompressionMethod::Zstd, Some(-7)),
        8 => (zip::CompressionMethod::Zstd, Some(3)),
        9 => (zip::CompressionMethod::Zstd, Some(22)),
        _ => (zip::CompressionMethod::Deflated, Some(6)),
    }
}
