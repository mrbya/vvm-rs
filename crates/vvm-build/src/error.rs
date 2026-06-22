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

    /// The same HDL definition was configured more than once.
    #[error("HDL definition `{name}` was configured more than once")]
    DuplicateDefine {
        /// Duplicate definition name.
        name: String,
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

    /// Verilator returned a version string that could not be parsed.
    #[error("could not parse Verilator version from `{output}`")]
    InvalidVerilatorVersion {
        /// Unparseable command output.
        output: String,
    },

    /// The selected Verilator version is unsupported.
    #[error("Verilator {found} is unsupported; VVM requires at least {minimum}")]
    UnsupportedVerilatorVersion {
        /// Detected version.
        found: String,

        /// Minimum supported version.
        minimum: String,
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

    /// Verilator did not produce an expected metadata output.
    #[error("Verilator metadata generation did not produce {role} `{path}`")]
    MissingMetadataOutput {
        /// Purpose of the expected output.
        role: &'static str,

        /// Expected metadata output path.
        path: PathBuf,
    },

    /// Verilator metadata output exists but is not a regular file.
    #[error("Verilator metadata generation produced non-file {role} `{path}`")]
    MetadataOutputNotFile {
        /// Purpose of the expected output.
        role: &'static str,

        /// Invalid metadata output path.
        path: PathBuf,
    },

    /// A generated metadata file could not be opened or read.
    #[error("failed to read {role} `{path}`")]
    MetadataRead {
        /// Metadata document role.
        role: &'static str,

        /// Metadata file path.
        path: PathBuf,

        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// A generated metadata file does not contain valid JSON.
    #[error("{role} `{path}` does not contain valid JSON")]
    InvalidMetadataJson {
        /// Metadata document role.
        role: &'static str,

        /// Metadata file path.
        path: PathBuf,

        /// JSON parser error, including line and column.
        #[source]
        source: serde_json::Error,
    },

    /// A metadata document root is not a JSON object.
    #[error("{role} `{path}` must contain a JSON object at its root")]
    MetadataRootNotObject {
        /// Metadata document role.
        role: &'static str,

        /// Metadata file path.
        path: PathBuf,
    },

    /// A required metadata field is missing.
    #[error("{role} `{path}` is missing required field `{field}`")]
    MissingMetadataField {
        /// Metadata document role.
        role: &'static str,

        /// Metadata file path.
        path: PathBuf,

        /// Missing field name.
        field: &'static str,
    },

    /// A metadata field has an unexpected JSON type.
    #[error("{role} `{path}` field `{field}` must be {expected}")]
    InvalidMetadataFieldType {
        /// Metadata document role.
        role: &'static str,

        /// Metadata file path.
        path: PathBuf,

        /// Invalid field name.
        field: &'static str,

        /// Expected JSON type.
        expected: &'static str,
    },

    /// The requested top module was not present in Verilator metadata.
    #[error("top module `{top_module}` was not found in Verilator metadata `{path}`")]
    MissingTopModuleMetadata {
        /// Requested HDL top-module name.
        top_module: String,

        /// Main metadata path.
        path: PathBuf,
    },

    /// More than one metadata module matched the requested top module.
    #[error("multiple module nodes matched top module `{top_module}` in `{path}`")]
    DuplicateTopModuleMetadata {
        /// Requested HDL top-module name.
        top_module: String,

        /// Main metadata path.
        path: PathBuf,
    },

    /// An AST address occurred more than once.
    #[error("metadata address `{address}` occurred more than once in `{path}`")]
    DuplicateMetadataAddress {
        /// Duplicate short AST address.
        address: String,

        /// Main metadata path.
        path: PathBuf,
    },

    /// An AST pointer could not be resolved.
    #[error("port `{port}` references unknown datatype `{reference}` in `{path}`")]
    UnresolvedPortDataType {
        /// HDL port name.
        port: String,

        /// Unresolved `dtypep` value.
        reference: String,

        /// Main metadata path.
        path: PathBuf,
    },

    /// A top-level port name occurred more than once.
    #[error("top module `{top_module}` contains duplicate port `{port}`")]
    DuplicatePortName {
        /// HDL top-module name.
        top_module: String,

        /// Duplicate HDL port name.
        port: String,
    },

    /// A port direction was not recognized.
    #[error("port `{port}` has unknown Verilator direction `{direction}`")]
    UnknownPortDirection {
        /// HDL port name.
        port: String,

        /// Unrecognized direction.
        direction: String,
    },

    /// A packed datatype range could not be interpreted.
    #[error("port `{port}` has unsupported packed range `{range}`")]
    InvalidPortRange {
        /// HDL port name.
        port: String,

        /// Unparseable range.
        range: String,
    },

    /// The referenced datatype kind is not currently supported.
    #[error("port `{port}` uses unsupported Verilator datatype `{kind}`")]
    UnsupportedPortDataType {
        /// HDL port name.
        port: String,

        /// Verilator AST datatype kind.
        kind: String,
    },

    /// Bidirectional ports are not currently supported.
    #[error("port `{port}` is bidirectional; inout ports are not supported")]
    UnsupportedInoutPort {
        /// HDL port name.
        port: String,
    },

    /// Signed ports are not currently supported.
    #[error("port `{port}` is signed; signed ports are not supported")]
    UnsupportedSignedPort {
        /// HDL port name.
        port: String,
    },

    /// Port width exceeds the current VVM limit.
    #[error("port `{port}` is {width} bits wide; the current maximum is {maximum}")]
    UnsupportedPortWidth {
        /// HDL port name.
        port: String,

        /// Actual packed width.
        width: u32,

        /// Current supported maximum.
        maximum: u32,
    },

    /// A metadata name cannot be represented by the initial generator.
    #[error("cannot generate {role} from `{name}`: {reason}")]
    UnsupportedCodegenName {
        /// Role of the problematic name.
        role: &'static str,

        /// Original metadata name.
        name: String,

        /// Reason generation is impossible.
        reason: &'static str,
    },

    /// Two generated adapter operations would use the same C++ name.
    #[error("generated C++ member name `{name}` is produced more than once")]
    GeneratedNameCollision {
        /// Ambiguous generated name.
        name: String,
    },
}
