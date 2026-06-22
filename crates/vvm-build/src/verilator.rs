use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fmt, fs};

use crate::builder::Define;
use crate::{BuildError, BuildResult, command};

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
        .arg(model.output_dir)
        .arg("--emit-accessors");

    for include_dir in model.hdl_include_dirs {
        command.arg(include_argument(include_dir));
    }

    for define in model.defines {
        command.arg(define_argument(define));
    }

    for argument in model.extra_arguments {
        command.arg(argument);
    }

    for source in model.sources {
        command.arg(source);
    }

    command
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
