//! Safe output-root preparation and atomic rendered-report writes.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use vvm_core::{CoverageMerge, CoverageReport};

use crate::error::CoverageCommandError;
use crate::metadata::WorkspaceMetadata;

/// Process-local suffix source for collision-resistant paths.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Complete filesystem layout for one coverage run.
#[derive(Debug, Clone)]
pub struct CoverageOutputLayout {
    /// Absolute output root.
    pub(crate) root: PathBuf,

    /// Absolute per-test artifact directory.
    pub(crate) artifacts: PathBuf,

    /// Merged JSON destination.
    pub(crate) merged: PathBuf,

    /// Text report destination.
    pub(crate) text: PathBuf,

    /// HTML report destination.
    pub(crate) html: PathBuf,
}

/// Validates a portable output filename base.
///
/// # Errors
///
/// Returns an error when the name cannot be used as a portable basename.
pub fn validate_name(value: &str) -> Result<(), CoverageCommandError> {
    let mut characters = value.chars();

    let Some(first) = characters.next() else {
        return Err(CoverageCommandError::InvalidOutputName {
            value: value.to_owned(),
        });
    };

    if !first.is_ascii_alphanumeric()
        || !characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        return Err(CoverageCommandError::InvalidOutputName {
            value: value.to_owned(),
        });
    }

    Ok(())
}

impl CoverageOutputLayout {
    /// Creates an isolated output root and all fixed child paths.
    pub(crate) fn create(
        output: Option<PathBuf>,
        name: &str,
        metadata: &WorkspaceMetadata,
    ) -> Result<Self, CoverageCommandError> {
        validate_name(name)?;

        let current = std::env::current_dir().map_err(|source| {
            CoverageCommandError::CreateOutputDirectory {
                path: PathBuf::from("."),
                source,
            }
        })?;

        let root = match output {
            Some(path) if path.is_absolute() => path,
            Some(path) => current.join(path),
            None => metadata
                .target_directory
                .join("vvm-coverage")
                .join(run_name()?),
        };

        prepare_root(&root)?;

        let root = fs::canonicalize(&root).map_err(|source| {
            CoverageCommandError::CreateOutputDirectory {
                path: root.clone(),
                source,
            }
        })?;

        let artifacts = root.join("artifacts");

        fs::create_dir_all(&artifacts).map_err(|source| {
            CoverageCommandError::CreateOutputDirectory {
                path: artifacts.clone(),
                source,
            }
        })?;

        let merged = root.join(format!("{name}{}", CoverageMerge::FILE_SUFFIX));
        let text = root.join(format!("{name}{}", CoverageReport::TEXT_FILE_SUFFIX));
        let html = root.join(format!("{name}{}", CoverageReport::HTML_FILE_SUFFIX));

        Ok(Self {
            root,
            artifacts,
            merged,
            text,
            html,
        })
    }
}

/// Creates only an absent root or accepts an existing empty directory.
fn prepare_root(root: &Path) -> Result<(), CoverageCommandError> {
    match fs::metadata(root) {
        Ok(metadata) if metadata.is_file() => Err(CoverageCommandError::OutputPathIsFile {
            path: root.to_owned(),
        }),

        Ok(_) => {
            let mut entries = fs::read_dir(root).map_err(|source| {
                CoverageCommandError::CreateOutputDirectory {
                    path: root.to_owned(),
                    source,
                }
            })?;

            if entries.next().is_some() {
                return Err(CoverageCommandError::OutputDirectoryNotEmpty {
                    path: root.to_owned(),
                });
            }

            Ok(())
        }

        Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir_all(root)
            .map_err(|create_error| CoverageCommandError::CreateOutputDirectory {
                path: root.to_owned(),
                source: create_error,
            }),

        Err(source) => Err(CoverageCommandError::CreateOutputDirectory {
            path: root.to_owned(),
            source,
        }),
    }
}

/// Returns a collision-resistant non-persisted run directory name.
fn run_name() -> Result<String, CoverageCommandError> {
    let milliseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| CoverageCommandError::CreateOutputDirectory {
            path: PathBuf::from("time"),
            source: std::io::Error::other(source),
        })?
        .as_millis();

    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);

    let name = format!("run-{milliseconds}-pid-{}-{sequence}", std::process::id());

    Ok(name)
}

/// Writes a new report atomically without replacing an existing final destination.
pub(crate) fn write_atomic_new(path: &Path, contents: &[u8]) -> Result<(), CoverageCommandError> {
    if path.exists() {
        return Err(CoverageCommandError::WriteReport {
            path: path.to_owned(),
            source: std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "report destination exists",
            ),
        });
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoverageCommandError::WriteReport {
            path: path.to_owned(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "report has no UTF-8 filename",
            ),
        })?;

    let temporary = path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));

    // Keep every write phase fallible before exposing the destination by rename.
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| CoverageCommandError::WriteReport {
                path: path.to_owned(),
                source,
            })?;

        file.write_all(contents)
            .map_err(|source| CoverageCommandError::WriteReport {
                path: path.to_owned(),
                source,
            })?;

        file.flush()
            .map_err(|source| CoverageCommandError::WriteReport {
                path: path.to_owned(),
                source,
            })?;

        file.sync_all()
            .map_err(|source| CoverageCommandError::WriteReport {
                path: path.to_owned(),
                source,
            })?;

        fs::rename(&temporary, path).map_err(|source| CoverageCommandError::WriteReport {
            path: path.to_owned(),
            source,
        })
    })();

    if result.is_err() {
        drop(fs::remove_file(&temporary));
    }

    result
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{CoverageOutputLayout, validate_name, write_atomic_new};
    use crate::error::CoverageCommandError;
    use crate::metadata::WorkspaceMetadata;

    #[test]
    fn validates_portable_output_names() {
        for name in ["coverage", "coverage-1", "coverage_1.2"] {
            assert!(validate_name(name).is_ok(), "{name}");
        }

        for name in [
            "",
            ".coverage",
            "coverage/name",
            "coverage name",
            "coverage!",
        ] {
            assert!(validate_name(name).is_err(), "{name}");
        }
    }

    #[test]
    fn creates_isolated_layout_with_requested_root() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let root = directory.path().join("coverage");
        let metadata = WorkspaceMetadata {
            target_directory: directory.path().join("target"),
        };

        let layout = CoverageOutputLayout::create(Some(root.clone()), "report", &metadata)?;

        assert_eq!(layout.root, fs::canonicalize(root)?);
        assert!(layout.artifacts.is_dir());
        assert_eq!(
            layout.merged.file_name().and_then(|name| name.to_str()),
            Some("report.vvmcov-merged.json")
        );
        assert_eq!(
            layout.text.file_name().and_then(|name| name.to_str()),
            Some("report.vvmcov.txt")
        );
        assert_eq!(
            layout.html.file_name().and_then(|name| name.to_str()),
            Some("report.vvmcov.html")
        );

        Ok(())
    }

    #[test]
    fn rejects_existing_file_or_nonempty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let metadata = WorkspaceMetadata {
            target_directory: directory.path().join("target"),
        };
        let file = directory.path().join("file");
        let nonempty = directory.path().join("nonempty");
        fs::write(&file, "existing")?;
        fs::create_dir_all(&nonempty)?;
        fs::write(nonempty.join("existing"), "existing")?;

        assert!(matches!(
            CoverageOutputLayout::create(Some(file), "report", &metadata),
            Err(CoverageCommandError::OutputPathIsFile { .. })
        ));
        assert!(matches!(
            CoverageOutputLayout::create(Some(nonempty), "report", &metadata),
            Err(CoverageCommandError::OutputDirectoryNotEmpty { .. })
        ));

        Ok(())
    }

    #[test]
    fn writes_once_without_replacing_destination() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let report = directory.path().join("report.txt");

        write_atomic_new(&report, b"first")?;

        assert_eq!(fs::read(&report)?, b"first");
        assert!(matches!(
            write_atomic_new(&report, b"second"),
            Err(CoverageCommandError::WriteReport { .. })
        ));
        assert_eq!(fs::read(&report)?, b"first");

        Ok(())
    }
}
