use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{BuildError, BuildResult};

/// Resolves a user path relative to the consuming package.
#[must_use]
pub fn resolve_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

/// Resolves and validates a configured file path.
pub fn resolve_file(manifest_dir: &Path, path: &Path, role: &'static str) -> BuildResult<PathBuf> {
    let resolved_path = resolve_path(manifest_dir, path);

    if !resolved_path.exists() {
        return Err(BuildError::MissingConfiguredPath {
            role,
            path: resolved_path,
        });
    }

    if !resolved_path.is_file() {
        return Err(BuildError::ConfiguredPathNotFile {
            role,
            path: resolved_path,
        });
    }

    resolved_path
        .canonicalize()
        .map_err(|source| BuildError::Io {
            operation: "canonicalize configured file path",
            path: resolved_path,
            source,
        })
}

/// Resolves and validates a configured directory path.
pub fn resolve_directory(
    manifest_dir: &Path,
    path: &Path,
    role: &'static str,
) -> BuildResult<PathBuf> {
    let resolved_path = resolve_path(manifest_dir, path);

    if !resolved_path.exists() {
        return Err(BuildError::MissingConfiguredPath {
            role,
            path: resolved_path,
        });
    }

    if !resolved_path.is_dir() {
        return Err(BuildError::ConfiguredPathNotDirectory {
            role,
            path: resolved_path,
        });
    }

    resolved_path
        .canonicalize()
        .map_err(|source| BuildError::Io {
            operation: "canonicalize configured directory path",
            path: resolved_path,
            source,
        })
}

/// Resolves multiple configured file paths.
pub fn resolve_files(
    manifest_dir: &Path,
    paths: &[PathBuf],
    role: &'static str,
) -> BuildResult<Vec<PathBuf>> {
    paths
        .iter()
        .map(|path| resolve_file(manifest_dir, path, role))
        .collect()
}

/// Resolves multiple configured directory paths.
pub fn resolve_directories(
    manifest_dir: &Path,
    paths: &[PathBuf],
    role: &'static str,
) -> BuildResult<Vec<PathBuf>> {
    paths
        .iter()
        .map(|path| resolve_directory(manifest_dir, path, role))
        .collect()
}

/// Ensures canonical configured paths are unique.
pub fn ensure_unique_paths(paths: &[PathBuf], role: &'static str) -> BuildResult<()> {
    let mut seen_paths = HashSet::new();

    for path in paths {
        if !seen_paths.insert(path.clone()) {
            return Err(BuildError::DuplicateConfiguredPath {
                role,
                path: path.clone(),
            });
        }
    }

    Ok(())
}
