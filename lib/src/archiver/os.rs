use std::path::Path;

use zip::write::SimpleFileOptions;

pub(super) mod windows;

pub(super) mod linux;

pub(super) fn create_file_opts(target: &Path, level: i64) -> SimpleFileOptions {
    if cfg!(target_os = "windows") {
        windows::create_file_opts(target, level)
    } else {
        linux::create_file_opts(target, level)
    }
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
