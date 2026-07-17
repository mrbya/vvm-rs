use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fmt, fs};

use crate::builder::Define;
use crate::trace::TraceFormat;
use crate::{BuildError, BuildResult, TraceOptions, command};

/// Default Verilator executable name.
const DEFAULT_EXECUTABLE: &str = "verilator";

/// Minimum supported Verilator version.
const MINIMUM_VERSION: VerilatorVersion = VerilatorVersion::new(5, 0);

/// Inputs needed to construct or run the Verilator model-generation command.
#[derive(Debug)]
pub struct ModelCommand<'a> {
    /// Verilator executable name or path.
    pub(crate) executable: &'a OsStr,
    /// DUT top module name.
    pub(crate) top_module: &'a str,
    /// Generated model prefix.
    pub(crate) model_prefix: &'a str,
    /// Output directory for generated Verilator artifacts.
    pub(crate) output_dir: &'a Path,
    /// HDL include directories.
    pub(crate) hdl_include_dirs: &'a [PathBuf],
    /// HDL preprocessor definitions.
    pub(crate) defines: &'a [Define],
    /// Additional raw Verilator arguments.
    pub(crate) extra_arguments: &'a [OsString],
    /// HDL source files.
    pub(crate) sources: &'a [PathBuf],
    /// Waveform trace config.
    pub(crate) trace: Option<TraceOptions>,
    /// Whether Verilator timing constructs are enabled.
    pub(crate) timing: bool,
    /// Whether Verilator must split top-level inouts into input, output-enable,
    /// and output-value members.
    pub(crate) inout_enables: bool,
}

/// Parsed Verilator version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VerilatorVersion {
    /// Major version component.
    major: u32,

    /// Minor version component.
    minor: u32,
}

impl VerilatorVersion {
    /// Creates a Verilator version.
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for VerilatorVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{:03}", self.major, self.minor)
    }
}

/// Selects the Verilator executable with builder override precedence.
#[must_use]
pub fn select_executable(configured: Option<OsString>, environment: Option<OsString>) -> OsString {
    configured
        .or(environment)
        .unwrap_or_else(|| OsString::from(DEFAULT_EXECUTABLE))
}

/// Returns the minimum supported Verilator version.
#[must_use]
#[cfg(test)]
pub const fn minimum_supported_version() -> VerilatorVersion {
    MINIMUM_VERSION
}

/// Builds an `-I<path>` HDL include argument.
#[must_use]
pub fn include_argument(path: &Path) -> OsString {
    let mut argument = OsString::from("-I");
    argument.push(path.as_os_str());
    argument
}

/// Builds a `-D<name>[=<value>]` HDL definition argument.
#[must_use]
pub fn define_argument(define: &Define) -> OsString {
    let mut argument = OsString::from("-D");
    argument.push(&define.name);

    if let Some(value) = define.value.as_ref() {
        argument.push("=");
        argument.push(value);
    }

    argument
}

/// Parses a Verilator version string.
pub fn parse_version(output: &str) -> BuildResult<VerilatorVersion> {
    let mut tokens = output.split_whitespace();

    let Some(program_name) = tokens.next() else {
        return Err(BuildError::InvalidVerilatorVersion {
            output: output.to_owned(),
        });
    };

    if program_name != "Verilator" {
        return Err(BuildError::InvalidVerilatorVersion {
            output: output.to_owned(),
        });
    }

    let Some(version_token) = tokens.next() else {
        return Err(BuildError::InvalidVerilatorVersion {
            output: output.to_owned(),
        });
    };

    let Some((major, minor)) = version_token.split_once('.') else {
        return Err(BuildError::InvalidVerilatorVersion {
            output: output.to_owned(),
        });
    };

    if minor.is_empty() {
        return Err(BuildError::InvalidVerilatorVersion {
            output: output.to_owned(),
        });
    }

    let major =
        major
            .parse::<u32>()
            .map_err(|_parse_error| BuildError::InvalidVerilatorVersion {
                output: output.to_owned(),
            })?;
    let minor =
        minor
            .parse::<u32>()
            .map_err(|_parse_error| BuildError::InvalidVerilatorVersion {
                output: output.to_owned(),
            })?;

    Ok(VerilatorVersion::new(major, minor))
}

/// Validates that the Verilator version meets the minimum supported version.
pub fn ensure_supported_version(version: VerilatorVersion) -> BuildResult<()> {
    if version < MINIMUM_VERSION {
        return Err(BuildError::UnsupportedVerilatorVersion {
            found: version.to_string(),
            minimum: MINIMUM_VERSION.to_string(),
        });
    }

    Ok(())
}

/// Queries and validates the Verilator version.
pub fn version(executable: &OsStr) -> BuildResult<VerilatorVersion> {
    let output = command::run(
        Command::new(executable).arg("--version"),
        "verilator version discovery",
    )?;

    let stdout =
        String::from_utf8(output.stdout).map_err(|source| BuildError::InvalidCommandOutput {
            context: "verilator version discovery",
            source,
        })?;

    parse_version(stdout.trim())
}

/// Queries Verilator installation root.
pub fn root(executable: &OsStr) -> BuildResult<PathBuf> {
    let output = command::run(
        Command::new(executable)
            .arg("--getenv")
            .arg("VERILATOR_ROOT"),
        "verilator root discovery",
    )?;

    let stdout =
        String::from_utf8(output.stdout).map_err(|source| BuildError::InvalidCommandOutput {
            context: "verilator root discovery",
            source,
        })?;

    let root = stdout.trim();

    if root.is_empty() {
        return Err(BuildError::EmptyVerilatorRoot);
    }

    Ok(PathBuf::from(root))
}

/// Builds the Verilator model-generation command.
#[must_use]
pub fn model_command(model: &ModelCommand<'_>) -> Command {
    let mut command = Command::new(model.executable);

    command
        .arg("--cc")
        .arg("--top-module")
        .arg(model.top_module)
        .arg("--prefix")
        .arg(model.model_prefix)
        .arg("--Mdir")
        .arg(model.output_dir);

    append_trace_arguments(&mut command, model.trace);
    append_timing_argument(&mut command, model.timing);
    append_inout_arguments(&mut command, model.inout_enables);

    append_hdl_arguments(
        &mut command,
        model.hdl_include_dirs,
        model.defines,
        model.extra_arguments,
        model.sources,
    );

    command
}

/// Appends top-level split-inout model-generation arguments.
fn append_inout_arguments(command: &mut Command, inout_enables: bool) {
    if inout_enables {
        command.arg("--pins-inout-enables");
    }
}

/// Invokes Verilator to generate a C++ model.
pub fn generate(model: &ModelCommand<'_>) -> BuildResult<()> {
    let mut command = model_command(model);

    command::run(&mut command, "verilator model generation")?;
    Ok(())
}

/// Finds Verilator-generated C++ translation units.
pub fn generated_sources(output_dir: &Path, model_prefix: &str) -> BuildResult<Vec<PathBuf>> {
    let read_dir = fs::read_dir(output_dir).map_err(|source| BuildError::Io {
        operation: "read generated source directory",
        path: output_dir.to_path_buf(),
        source,
    })?;

    let combined_sources = format!("{model_prefix}__ALL.cpp");
    let mut sources = Vec::new();

    for entry_result in read_dir {
        let entry = entry_result.map_err(|source| BuildError::Io {
            operation: "read generated source directory entry",
            path: output_dir.to_path_buf(),
            source,
        })?;

        let path = entry.path();
        let is_cpp = path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("cpp"));
        let is_combined_source = path
            .file_name()
            .is_some_and(|name| name == OsStr::new(&combined_sources));

        if is_cpp && !is_combined_source {
            sources.push(path);
        }
    }

    sources.sort();

    if sources.is_empty() {
        return Err(BuildError::NoGeneratedSources {
            path: output_dir.to_path_buf(),
        });
    }

    Ok(sources)
}

/// Inputs needed to construct and/or run  Verilator metadata generation.
#[derive(Debug)]
pub struct MetadataCommand<'a> {
    /// Verilator executable name or path.
    pub executable: &'a OsStr,

    /// HDL top module name.
    pub top_module: &'a str,

    /// Main `.tree.json` output path.
    pub output: &'a Path,

    /// Companion `.tree.meta.json` output path.
    pub meta_output: &'a Path,

    /// HDL include directories.
    pub hdl_include_dirs: &'a [PathBuf],

    /// HDL preprocessor definitions.
    pub defines: &'a [Define],

    /// Additional raw Verilator arguments.
    pub extra_arguments: &'a [OsString],

    /// HDL source files.
    pub sources: &'a [PathBuf],

    /// Whether Verilator timing constructs are enabled.
    pub timing: bool,
}

/// Files produced by Verilator metadata generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataFiles {
    /// Main AST json file.
    pub tree: PathBuf,
    /// Companion metadata json file.
    pub meta: PathBuf,
}

/// Builds the Verilator JSON metadata-generation command.
#[must_use]
pub fn metadata_command(metadata: &MetadataCommand<'_>) -> Command {
    let mut command = Command::new(metadata.executable);

    command
        .arg("--json-only")
        .arg("--json-only-output")
        .arg(metadata.output)
        .arg("--json-only-meta-output")
        .arg(metadata.meta_output)
        .arg("--no-json-edit-nums")
        .arg("--top-module")
        .arg(metadata.top_module);

    append_timing_argument(&mut command, metadata.timing);

    append_hdl_arguments(
        &mut command,
        metadata.hdl_include_dirs,
        metadata.defines,
        metadata.extra_arguments,
        metadata.sources,
    );

    command
}

/// Invokes Verilator to generate JSON metadata.
///
/// # Errors
///
/// Returns [`BuildError`] if the command fails or either expected
/// metadata output is not produced as a regular file.
pub fn generate_metadata(metadata: &MetadataCommand<'_>) -> BuildResult<MetadataFiles> {
    remove_stale_metadata_output(metadata.output)?;
    remove_stale_metadata_output(metadata.meta_output)?;

    let mut command = metadata_command(metadata);

    command::run(&mut command, "verilator metadata generation")?;

    ensure_metadata_output(metadata.output, "AST metadata output")?;

    ensure_metadata_output(metadata.meta_output, "AST file metadata output")?;

    Ok(MetadataFiles {
        tree: metadata.output.to_path_buf(),
        meta: metadata.meta_output.to_path_buf(),
    })
}

/// Verifies that Verilator produced a regular metadata file.
pub fn ensure_metadata_output(path: &Path, role: &'static str) -> BuildResult<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(BuildError::MissingMetadataOutput {
                role,
                path: path.to_path_buf(),
            });
        }
        Err(source) => {
            return Err(BuildError::Io {
                operation: "inspect generated metadata output",
                path: path.to_path_buf(),
                source,
            });
        }
    };

    if !metadata.is_file() {
        return Err(BuildError::MetadataOutputNotFile {
            role,
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

/// Removes an existing regular metadata output file.
fn remove_stale_metadata_output(path: &Path) -> BuildResult<()> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(source) => {
            return Err(BuildError::Io {
                operation: "inspect stale metadata output",
                path: path.to_path_buf(),
                source,
            });
        }
    };

    // Leave non-file paths in place. The command or the postcondition
    // validation will then produce a meaningful failure.
    if !metadata.is_file() {
        return Ok(());
    }

    fs::remove_file(path).map_err(|source| BuildError::Io {
        operation: "remove stale metadata output",
        path: path.to_path_buf(),
        source,
    })
}

/// Appends shared HDL elaboration arguments to a Verilator command.
fn append_hdl_arguments(
    command: &mut Command,
    hdl_include_dirs: &[PathBuf],
    defines: &[Define],
    extra_arguments: &[OsString],
    sources: &[PathBuf],
) {
    for include_dir in hdl_include_dirs {
        command.arg(include_argument(include_dir));
    }

    for define in defines {
        command.arg(define_argument(define));
    }

    for argument in extra_arguments {
        command.arg(argument);
    }

    for source in sources {
        command.arg(source);
    }
}

/// Appends native waveform tracing flags to a Verilator command.
fn append_trace_arguments(command: &mut Command, trace: Option<TraceOptions>) {
    let Some(TraceOptions { format, depth }) = trace else {
        return;
    };

    match format {
        TraceFormat::Vcd => command.arg("--trace"),
        TraceFormat::Fst => command.arg("--trace-fst"),
    };

    command.arg("--trace-depth");
    command.arg(depth.to_string());
}

/// Appends the typed timing mode before user-supplied raw arguments.
fn append_timing_argument(command: &mut Command, timing: bool) {
    if timing {
        command.arg("--timing");
    }
}
