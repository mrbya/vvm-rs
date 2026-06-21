use std::process::{Command, Output};

use crate::{BuildError, BuildResult};

/// Runs a command and returns its captured output on success.
///
/// # Arguments
/// - `command`: os command to execute,
/// - `context`: command context/description.
///
/// # Returns
/// Command output if execution succeeds.
///
/// # Errors
/// Returns following errors:
/// - [`BuildError::CommandStart`] if a command fails to start,
/// - [`BuildError::CommandFailed`] if a command fails during execution.
pub fn run(command: &mut Command, context: &'static str) -> BuildResult<Output> {
    let rendered_command = format!("{command:?}");

    let output = command
        .output()
        .map_err(|source| BuildError::CommandStart {
            context,
            command: rendered_command.clone(),
            source,
        })?;

    if output.status.success() {
        return Ok(output);
    }

    Err(BuildError::CommandFailed {
        context,
        command: rendered_command,
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
