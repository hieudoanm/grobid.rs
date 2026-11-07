//! Version.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

/// Get a version string based on file modification time (matching v1 behavior).
pub fn get_version() -> String {
    // Try to get mtime of the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(meta) = fs::metadata(&exe) {
            if let Ok(mtime) = meta.modified() {
                let dt: DateTime<Local> = mtime.into();
                return dt.format("%Y.%m.%d.%H.%M.%S").to_string();
            }
        }
    }
    // Try to get mtime of this source file if running from source
    let src_path = Path::new(file!());
    if let Ok(canon) = fs::canonicalize(src_path) {
        if let Ok(meta) = fs::metadata(&canon) {
            if let Ok(mtime) = meta.modified() {
                let dt: DateTime<Local> = mtime.into();
                return dt.format("%Y.%m.%d.%H.%M.%S").to_string();
            }
        }
    }
    // Fallback to current time
    Local::now().format("%Y.%m.%d.%H.%M.%S").to_string()
}
