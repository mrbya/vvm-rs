use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::string::FromUtf8Error;

use thiserror::Error;

/// Result alias used by `vvm-build`.
pub type BuildResult<T> = std::result::Result<T, BuildError>;

/// An error encountered while generating or compiling a Verilated DUT.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BuildError {
    /// DUT top module config missing.
    #[error("the DUT top module was not configured")]
    MissingTopModule,

    /// No HDL sources were provided in config.
    #[error("no HDL source files were configured")]
    MissingSources,

    /// No CXX bridge source was provided.
    #[error("no CXX bridge source was configured")]
    MissingBridge,

    /// Provided name cannot be used as a generated C++ identifier.
    #[error("{field} `{value}` is not a valid identifier.")]
    InvalidIdentifier {
        /// Name of the invalid field property.
        field: &'static str,

        /// Invalid value.
        value: String,
    },

    /// A configured path does not exist.
    #[error("configured {role} `{path}` does not exist")]
    MissingConfiguredPath {
        /// Role of the configured path.
        role: &'static str,

        /// Missing path.
        path: PathBuf,
    },

    /// A configured path is expected to be a file but is not.
    #[error("configured {role} `{path}` is not a file")]
    ConfiguredPathNotFile {
        /// Role of the configured path.
        role: &'static str,

        /// Non-file path.
        path: PathBuf,
    },

    /// A configured path is expected to be a directory but is not.
    #[error("configured {role} `{path}` is not a directory")]
    ConfiguredPathNotDirectory {
        /// Role of the configured path.
        role: &'static str,

        /// Non-directory path.
        path: PathBuf,
    },

    /// A configured path was added multiple times.
    #[error("configured {role} `{path}` was added more than once")]
    DuplicateConfiguredPath {
        /// Role of the configured path.
        role: &'static str,

        /// Duplicated path.
        path: PathBuf,
    },

    /// A required Cargo build env variable is absent.
    #[error("required env variable `{name}` was not set")]
    MissingEnvironmentVariable {
        /// Env variable name.
        name: &'static str,
    },

    /// Filesystem IO error.
    #[error("failed to {operation} `{path}`")]
    Io {
        /// Description of attempted operation.
        operation: &'static str,

        /// Path involved in the operation.
        path: PathBuf,

        /// Underlying io error.
        #[source]
        source: io::Error,
    },

    /// An external command failed to start.
    #[error("{context} failed to start for `{command}`")]
    CommandStart {
        /// Description of command.
        context: &'static str,

        /// Rendered command.
        command: String,

        /// Underlying process IO error.
        #[source]
        source: io::Error,
    },

    /// An external command failed.
    #[error(
        "{context} failed for `{command}`\nstatus: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    CommandFailed {
        /// Description of command.
        context: &'static str,

        /// Rendered command.
        command: String,

        /// Command exit status.
        status: ExitStatus,

        /// Captured standard output.
        stdout: String,

        /// Captured standard error.
        stderr: String,
    },

    /// Command output expected to contain UTF-8 only, but did not.
    #[error("{context} returned a non-valid utf-8 output")]
    InvalidCommandOutput {
        /// Command context/description.
        context: &'static str,

        /// UTF-8 conversion error.
        #[source]
        source: FromUtf8Error,
    },

    /// Verilator returned an empty installation root.
    #[error("Verilator returned an empty VERILATOR_ROOT")]
    EmptyVerilatorRoot,

    /// Verilator generated no model translation units.
    #[error("Verilator generated no C++ sources under `{path}`")]
    NoGeneratedSources {
        /// Expected generated-source directory.
        path: PathBuf,
    },

    /// A required Verilator runtime source was not found.
    #[error("required Verilator runtime source at `{path}` was not found")]
    MissingRuntimeSource {
        /// Missing runtime source path.
        path: PathBuf,
    },
}
