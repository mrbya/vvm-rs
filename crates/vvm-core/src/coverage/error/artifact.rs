use std::fmt;
use std::path::PathBuf;

use crate::CoverageDefinitionFingerprint;

/// Filesystem operation involved in coverage persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CoverageIoOperation {
    /// Create the output directory.
    CreateDirectory,
    /// Create a same-directory temporary file.
    CreateTemporaryFile,
    /// Write artifact bytes.
    Write,
    /// Synchronize artifact bytes.
    Synchronize,
    /// Rename temporary file to final destination.
    Rename,
    /// Read an artifact file.
    Read,
}

impl fmt::Display for CoverageIoOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match *self {
            Self::CreateDirectory => "create directory",
            Self::CreateTemporaryFile => "create temporary file",
            Self::Write => "write",
            Self::Synchronize => "synchronize",
            Self::Rename => "rename",
            Self::Read => "read",
        })
    }
}

/// Failure while encoding, decoding, validating, or persisting an artifact.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CoveragePersistenceError {
    /// JSON encoding failed.
    #[error("could not encode coverage artifact JSON: {source}")]
    JsonEncode {
        /// Underlying JSON encoder error.
        source: serde_json::Error,
    },
    /// JSON decoding failed.
    #[error("could not decode coverage artifact JSON: {source}")]
    JsonDecode {
        /// Underlying JSON decoder error.
        source: serde_json::Error,
    },
    /// Artifact format was not recognized.
    #[error("unsupported coverage artifact format `{found}`")]
    InvalidFormat {
        /// Unrecognized format name.
        found: String,
    },
    /// Artifact schema version is unsupported.
    #[error(
        "unsupported coverage artifact schema version {found}; supported version is {supported}"
    )]
    UnsupportedSchemaVersion {
        /// Encountered schema version.
        found: u32,
        /// Supported schema version.
        supported: u32,
    },
    /// Fingerprint string was invalid.
    #[error("invalid coverage definition fingerprint `{value}`")]
    InvalidFingerprint {
        /// Invalid fingerprint string.
        value: String,
    },
    /// Stored fingerprint disagreed with computed definition.
    #[error(
        "coverage fingerprint mismatch for instance `{instance_path}`: stored {stored}, computed \
         {computed}"
    )]
    FingerprintMismatch {
        /// Group instance path.
        instance_path: String,
        /// Stored fingerprint.
        stored: CoverageDefinitionFingerprint,
        /// Recomputed fingerprint.
        computed: CoverageDefinitionFingerprint,
    },
    /// Numeric conversion could not be represented.
    #[error("coverage artifact numeric overflow for `{field}`")]
    NumericOverflow {
        /// Numeric field that overflowed.
        field: String,
    },
    /// JSON data violated an artifact invariant.
    #[error("invalid coverage artifact data at `{path}`: {reason}")]
    InvalidData {
        /// Logical document path.
        path: String,
        /// Invariant failure reason.
        reason: String,
    },
    /// Final destination already exists.
    #[error("coverage artifact destination `{}` already exists", path.display())]
    DestinationExists {
        /// Existing final destination.
        path: PathBuf,
    },
    /// Filesystem operation failed.
    #[error("could not {operation} coverage artifact `{}`: {source}", path.display())]
    Io {
        /// Failed filesystem operation.
        operation: CoverageIoOperation,
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CoverageIoOperation, CoveragePersistenceError};

    #[test]
    fn io_operations_have_stable_human_readable_names() {
        let operations = [
            (CoverageIoOperation::CreateDirectory, "create directory"),
            (
                CoverageIoOperation::CreateTemporaryFile,
                "create temporary file",
            ),
            (CoverageIoOperation::Write, "write"),
            (CoverageIoOperation::Synchronize, "synchronize"),
            (CoverageIoOperation::Rename, "rename"),
            (CoverageIoOperation::Read, "read"),
        ];

        for (operation, expected) in operations {
            assert_eq!(operation.to_string(), expected);
        }
    }

    #[test]
    fn persistence_errors_preserve_the_failed_path_and_context() {
        let path = PathBuf::from("coverage/result.json");
        let error = CoveragePersistenceError::Io {
            operation: CoverageIoOperation::Write,
            path: path.clone(),
            source: std::io::Error::from_raw_os_error(28),
        };

        assert_eq!(
            error.to_string(),
            "could not write coverage artifact `coverage/result.json`: No space left on device \
             (os error 28)"
        );
        assert_eq!(
            CoveragePersistenceError::DestinationExists { path }.to_string(),
            "coverage artifact destination `coverage/result.json` already exists"
        );
    }
}
