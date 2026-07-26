//! Structured failures emitted by coverage orchestration.

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error;
use vvm_core::{CoverageMergeError, CoveragePersistenceError};

/// Failure while preparing or post-processing a coverage command.
#[derive(Debug, Error)]
pub enum CoverageCommandError {
    /// The supplied command is not a supported Cargo test command.
    #[error("unsupported coverage child command: {command:?}; use `test ...` or `nextest run ...`")]
    UnsupportedChildCommand {
        /// Rejected forwarded command.
        command: Vec<OsString>,
    },

    /// The forwarded command redundantly starts with Cargo.
    #[error("do not include `cargo` after `--`; pass the Cargo subcommand directly")]
    CargoArgumentAfterSeparator,

    /// A split manifest path has no value.
    #[error("`--manifest-path` requires a following path")]
    MissingManifestPathValue,

    /// A nextest retry value could not be accepted.
    #[error(
        "nextest retries are unsupported because VVM coverage artifacts do not identify \
         individual retry attempts: {value:?}"
    )]
    UnsupportedNextestRetries {
        /// Rejected retry value.
        value: OsString,
    },

    /// Metadata could not be launched.
    #[error("failed to run Cargo metadata with `{program:?}`: {source}")]
    MetadataSpawn {
        /// Cargo executable.
        program: OsString,
        #[source]
        /// Spawn failure.
        source: io::Error,
    },

    /// Cargo metadata returned a failure.
    #[error("Cargo metadata failed with {status}: {stderr}")]
    MetadataFailed {
        /// Metadata process status.
        status: ExitStatus,
        /// Captured Cargo diagnostic.
        stderr: String,
    },

    /// Metadata JSON did not match Cargo's stable format.
    #[error("failed to decode Cargo metadata: {source}")]
    MetadataDecode {
        #[source]
        /// JSON decoding failure.
        source: serde_json::Error,
    },

    /// An output filename base is not portable or safe.
    #[error("invalid coverage output name `{value}`")]
    InvalidOutputName {
        /// Invalid base name.
        value: String,
    },

    /// The requested output root is a regular file.
    #[error("coverage output path is a file: `{path}`")]
    OutputPathIsFile {
        /// Conflicting file path.
        path: PathBuf,
    },

    /// The requested output root would reuse previous contents.
    #[error("coverage output directory is not empty: `{path}`")]
    OutputDirectoryNotEmpty {
        /// Reused nonempty directory.
        path: PathBuf,
    },

    /// A coverage output directory could not be created.
    #[error("failed to create coverage output directory `{path}`: {source}")]
    CreateOutputDirectory {
        /// Directory path.
        path: PathBuf,
        #[source]
        /// Filesystem failure.
        source: io::Error,
    },

    /// The test child could not be spawned.
    #[error("failed to spawn coverage child {command:?}: {source}")]
    SpawnChild {
        /// Complete child command.
        command: Vec<OsString>,
        #[source]
        /// Spawn failure.
        source: io::Error,
    },

    /// The artifact directory could not be enumerated.
    #[error("failed to read coverage artifact directory `{path}`: {source}")]
    ReadArtifactDirectory {
        /// Artifact directory.
        path: PathBuf,
        #[source]
        /// Directory error.
        source: io::Error,
    },

    /// An artifact-directory entry could not be inspected.
    #[error("failed to read a coverage artifact entry in `{path}`: {source}")]
    ReadArtifactEntry {
        /// Artifact directory.
        path: PathBuf,
        #[source]
        /// Entry inspection error.
        source: io::Error,
    },

    /// No per-test coverage documents were produced.
    #[error("no per-test coverage artifacts found in `{path}`")]
    NoArtifacts {
        /// Searched artifact directory.
        path: PathBuf,
    },

    /// Existing core merging rejected the selected inputs.
    #[error(transparent)]
    Merge {
        #[from]
        /// Core merge failure.
        source: CoverageMergeError,
    },

    /// Existing core persistence rejected a merged document.
    #[error(transparent)]
    Persistence {
        #[from]
        /// Core persistence failure.
        source: CoveragePersistenceError,
    },

    /// A report could not be atomically written.
    #[error("failed to write report `{path}`: {source}")]
    WriteReport {
        /// Report destination.
        path: PathBuf,
        #[source]
        /// Write failure.
        source: io::Error,
    },

    /// Standard output could not accept the text report.
    #[error("failed to write report to stdout: {source}")]
    WriteStdout {
        #[source]
        /// Standard output failure.
        source: io::Error,
    },
}
