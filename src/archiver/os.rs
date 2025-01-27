use std::path::PathBuf;

use zip::write::SimpleFileOptions;

pub(super) mod windows;

pub(super) mod linux;

pub(super) fn create_file_opts(target: &PathBuf) -> SimpleFileOptions {
    if cfg!(target_os = "windows") {
        windows::create_file_opts(target)
    } else {
        linux::create_file_opts(target)
    }
}
