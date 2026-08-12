use std::fs::Metadata;
use std::path::Path;

use time::{OffsetDateTime, PrimitiveDateTime};
use zip::DateTime;
use zip::write::SimpleFileOptions;

use crate::{Error, Result};

pub(super) fn create_file_opts(target: &Path, level: i64) -> Result<SimpleFileOptions> {
    let metadata = std::fs::metadata(target).map_err(Error::IO)?;
    Ok(create_file_option(&metadata, level))
}

pub(crate) fn permission(metadata: &Metadata) -> u32 {
    #[cfg(target_os = "windows")]
    {
        let _ = metadata;
        0o644
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }
}

fn create_file_option(metadata: &Metadata, level: i64) -> SimpleFileOptions {
    let (method, level) = method_and_level(level);
    let opts = SimpleFileOptions::default()
        .compression_method(method)
        .compression_level(level);
    let opts = match last_modified_time(metadata) {
        Some(t) => opts.last_modified_time(t),
        // A timestamp outside the MS-DOS range zip can represent; the zip crate's
        // own default is used instead of failing the whole entry.
        None => opts,
    };
    #[cfg(target_os = "windows")]
    {
        opts
    }
    #[cfg(not(target_os = "windows"))]
    {
        opts.unix_permissions(permission(metadata))
    }
}

fn last_modified_time(metadata: &Metadata) -> Option<DateTime> {
    let modified = OffsetDateTime::from(metadata.modified().ok()?);
    DateTime::try_from(PrimitiveDateTime::new(modified.date(), modified.time())).ok()
}

/// Maps the totebag level (0-9) onto a zip compression method and its own level.
///
/// Levels 7-9 used to select Zstd, which is the one `zip` feature backed by a C
/// library. They now select Xz, which `zip` implements through the pure Rust
/// `lzma-rust2` and which compresses at least as well.
pub(crate) fn method_and_level(level: i64) -> (zip::CompressionMethod, Option<i64>) {
    match level {
        0 => (zip::CompressionMethod::Stored, None),
        1 => (zip::CompressionMethod::Deflated, Some(10)),
        2 => (zip::CompressionMethod::Deflated, Some(24)),
        3 => (zip::CompressionMethod::Deflated, Some(264)),
        4 => (zip::CompressionMethod::Bzip2, Some(1)),
        5 => (zip::CompressionMethod::Bzip2, Some(6)),
        6 => (zip::CompressionMethod::Bzip2, Some(9)),
        7 => (zip::CompressionMethod::Xz, Some(3)),
        8 => (zip::CompressionMethod::Xz, Some(6)),
        9 => (zip::CompressionMethod::Xz, Some(9)),
        _ => (zip::CompressionMethod::Deflated, Some(6)),
    }
}
