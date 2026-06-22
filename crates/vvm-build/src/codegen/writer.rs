use std::fs;
use std::path::Path;

use crate::{BuildError, BuildResult};

/// Result of writing one generated artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WriteOutcome {
    /// The artifact did not previously exist.
    Created,

    /// The artifact existed with different contents.
    Updated,

    /// The artifact already contained the requested bytes.
    Unchanged,
}

/// Writes a generated UTF-8 file only when its contents changed.
///
/// # Errors
///
/// Returns an I/O error when an existing file cannot be read or the new
/// contents cannot be written.
pub(super) fn write_if_changed(path: &Path, contents: &str) -> BuildResult<WriteOutcome> {
    let outcome = match fs::read(path) {
        Ok(existing) if existing.as_slice() == contents.as_bytes() => {
            return Ok(WriteOutcome::Unchanged);
        }
        Ok(_existing) => WriteOutcome::Updated,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => WriteOutcome::Created,
        Err(source) => {
            return Err(BuildError::Io {
                operation: "read generated source",
                path: path.to_path_buf(),
                source,
            });
        }
    };

    fs::write(path, contents).map_err(|source| BuildError::Io {
        operation: "write generated source",
        path: path.to_path_buf(),
        source,
    })?;

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{WriteOutcome, write_if_changed};

    #[test]
    fn creates_missing_generated_file() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let path = directory.path().join("generated.rs");

        assert_eq!(write_if_changed(&path, "first\n")?, WriteOutcome::Created);

        assert_eq!(fs::read_to_string(path)?, "first\n");

        Ok(())
    }

    #[test]
    fn leaves_identical_generated_file_unchanged() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let path = directory.path().join("generated.rs");

        assert_eq!(write_if_changed(&path, "same\n")?, WriteOutcome::Created);

        assert_eq!(write_if_changed(&path, "same\n")?, WriteOutcome::Unchanged);

        Ok(())
    }

    #[test]
    fn updates_changed_generated_file() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let path = directory.path().join("generated.rs");

        write_if_changed(&path, "before\n")?;

        assert_eq!(write_if_changed(&path, "after\n")?, WriteOutcome::Updated);

        assert_eq!(fs::read_to_string(path)?, "after\n");

        Ok(())
    }
}
