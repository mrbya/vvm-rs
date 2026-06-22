//! Deterministic generated-file writing.

use std::fs;
use std::path::Path;

use crate::{BuildError, BuildResult};

/// Writes a generated UTF-8 file only when its contents changed.
///
/// # Errors
///
/// Returns an I/O error when an existing file cannot be read or the new
/// contents cannot be written.
pub fn write_if_changed(path: &Path, contents: &str) -> BuildResult<()> {
    match fs::read(path) {
        Ok(existing) if existing.as_slice() == contents.as_bytes() => {
            return Ok(());
        }
        Ok(_existing) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(BuildError::Io {
                operation: "read generated source",
                path: path.to_path_buf(),
                source,
            });
        }
    }

    fs::write(path, contents).map_err(|source| BuildError::Io {
        operation: "write generated source",
        path: path.to_path_buf(),
        source,
    })
}
