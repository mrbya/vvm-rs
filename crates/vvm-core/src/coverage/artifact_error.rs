use std::fmt;
use std::path::PathBuf;

use crate::CoverageDefinitionFingerprint;

/// Filesystem operation involved in coverage persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug)]
pub enum CoveragePersistenceError {
    /// JSON encoding failed.
    JsonEncode {
        /// Underlying JSON encoder error.
        source: serde_json::Error,
    },
    /// JSON decoding failed.
    JsonDecode {
        /// Underlying JSON decoder error.
        source: serde_json::Error,
    },
    /// Artifact format was not recognized.
    InvalidFormat {
        /// Unrecognized format name.
        found: String,
    },
    /// Artifact schema version is unsupported.
    UnsupportedSchemaVersion {
        /// Encountered schema version.
        found: u32,
        /// Supported schema version.
        supported: u32,
    },
    /// Fingerprint string was invalid.
    InvalidFingerprint {
        /// Invalid fingerprint string.
        value: String,
    },
    /// Stored fingerprint disagreed with computed definition.
    FingerprintMismatch {
        /// Group instance path.
        instance_path: String,
        /// Stored fingerprint.
        stored: CoverageDefinitionFingerprint,
        /// Recomputed fingerprint.
        computed: CoverageDefinitionFingerprint,
    },
    /// Numeric conversion could not be represented.
    NumericOverflow {
        /// Numeric field that overflowed.
        field: String,
    },
    /// JSON data violated an artifact invariant.
    InvalidData {
        /// Logical document path.
        path: String,
        /// Invariant failure reason.
        reason: String,
    },
    /// Final destination already exists.
    DestinationExists {
        /// Existing final destination.
        path: PathBuf,
    },
    /// Filesystem operation failed.
    Io {
        /// Failed filesystem operation.
        operation: CoverageIoOperation,
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

impl fmt::Display for CoveragePersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::JsonEncode { ref source } => write!(
                formatter,
                "could not encode coverage artifact JSON: {source}"
            ),
            Self::JsonDecode { ref source } => write!(
                formatter,
                "could not decode coverage artifact JSON: {source}"
            ),
            Self::InvalidFormat { ref found } => {
                write!(formatter, "unsupported coverage artifact format `{found}`")
            }
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "unsupported coverage artifact schema version {found}; supported version is \
                 {supported}"
            ),
            Self::InvalidFingerprint { ref value } => write!(
                formatter,
                "invalid coverage definition fingerprint `{value}`"
            ),
            Self::FingerprintMismatch {
                ref instance_path,
                stored,
                computed,
            } => write!(
                formatter,
                "coverage fingerprint mismatch for instance `{instance_path}`: stored {stored}, \
                 computed {computed}"
            ),
            Self::NumericOverflow { ref field } => write!(
                formatter,
                "coverage artifact numeric overflow for `{field}`"
            ),
            Self::InvalidData {
                ref path,
                ref reason,
            } => write!(
                formatter,
                "invalid coverage artifact data at `{path}`: {reason}"
            ),
            Self::DestinationExists { ref path } => write!(
                formatter,
                "coverage artifact destination `{}` already exists",
                path.display()
            ),
            Self::Io {
                operation,
                ref path,
                ref source,
            } => write!(
                formatter,
                "could not {operation} coverage artifact `{}`: {source}",
                path.display()
            ),
        }
    }
}
impl std::error::Error for CoveragePersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::JsonEncode { ref source } | Self::JsonDecode { ref source } => Some(source),
            Self::Io { ref source, .. } => Some(source),
            _ => None,
        }
    }
}
