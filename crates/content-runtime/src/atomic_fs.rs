//! Atomic file helpers — never leave a half-written active pointer.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Write `data` to `path` via temp file + rename (same-directory).
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = tmp_path(path);
    fs::write(&tmp, data)?;
    // On Unix rename is atomic when dest is same filesystem.
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Write pretty JSON atomically.
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let text = serde_json::to_vec_pretty(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_write(path, &text)
}

/// Recursively remove directory if it exists.
pub fn remove_dir_all_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    path.with_file_name(format!(".{name}.{}.tmp", std::process::id()))
}
