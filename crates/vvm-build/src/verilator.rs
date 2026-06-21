use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use crate::{BuildError, BuildResult, command};

/// Default Verilator executable name.
const DEFAULT_EXECUTABLE: &str = "verilator";

/// Returns the configured Verilator executable.
#[must_use]
pub fn executable() -> OsString {
    env::var_os("VERILATOR").unwrap_or_else(|| OsString::from(DEFAULT_EXECUTABLE))
}

/// Invokes Verilator to generate a C++ model.
///
/// # Arguments
/// - `executable`: Verilator executable name,
/// - `top-module`: DUT top module,
/// - `model_prefix`: verilated top module prefix,
/// - `output_dir`: Verilator-generated source directory,
/// - `sources`: discovered Verilator-generated C++ translation units,
/// - `extra_arguments`: extra args passed to verilator executable.
///
/// # Returns
/// Ok(()) on successfull model generation.
///
/// # Errors
/// Returns [`BuildError`] if the underlying verilator command fails.
pub fn generate(
    executable: &OsStr,
    top_module: &str,
    model_prefix: &str,
    output_dir: &Path,
    sources: &[PathBuf],
    extra_arguments: &[OsString],
) -> BuildResult<()> {
    let mut command = Command::new(executable);

    command
        .arg("-cc")
        .arg("--top-module")
        .arg(top_module)
        .arg("--prefix")
        .arg(model_prefix)
        .arg("--Mdir")
        .arg(output_dir)
        .arg("--emit-accessors");

    for argument in extra_arguments {
        command.arg(argument);
    }

    for source in sources {
        command.arg(source);
    }

    command::run(&mut command, "verilator model generation")?;

    Ok(())
}

/// Queries Verilator installation root.
///
/// # Arguments
/// - `executable`: Verilator executable name.
///
/// # Returns
/// Path to Verilator installation root.
///
/// # Errors
/// Returns [`BuildError`] if:
/// - root discovery command fails,
/// - Verilator installation root is empty.
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
        return Err(BuildError::EptyVerilatorRoot);
    }

    Ok(PathBuf::from(root))
}

/// Finds Verilator-generated C++ translation units
///
/// # Arguments
/// - `output-dir`: Verilator-generated source directory,
/// - `model_prefix`: Verilated top module prefix.
///
/// # Returns
/// Vector of verilator-generated C++ translation unit paths.
///
/// # Errors
/// Returns [`BuildError`] if any of the underlying IO operations failed or no Verilator-generated
/// sources found.
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

        let is_cpp = path.extension().is_some_and(|ext| ext == OsStr::new("cpp"));

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
