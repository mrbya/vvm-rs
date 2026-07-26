//! Validation of supported forwarded Cargo test commands.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use crate::error::CoverageCommandError;

/// Supported test execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCommandKind {
    /// Cargo's standard test runner.
    CargoTest,
    /// Cargo-nextest's test runner.
    NextestRun,
}

/// Parsed shape of a forwarded Cargo command without rewriting its arguments.
#[derive(Debug, Clone)]
pub struct CargoInvocation {
    /// Cargo executable selected from the environment.
    pub(crate) cargo: OsString,

    /// Optional immediate Cargo toolchain selector.
    pub(crate) toolchain: Option<OsString>,

    /// Cargo arguments, starting at the subcommand.
    pub(crate) arguments: Vec<OsString>,

    /// Test command mode.
    pub(crate) kind: TestCommandKind,

    /// Forwarded Cargo manifest path when present.
    pub(crate) manifest_path: Option<PathBuf>,
}

/// Parses only the pieces of the forwarded command required by orchestration.
///
/// # Errors
///
/// Returns an error when the command is unsupported or contains invalid options.
pub fn parse(
    child: Vec<OsString>,
    cargo: OsString,
) -> Result<CargoInvocation, CoverageCommandError> {
    let arguments = if child.is_empty() {
        vec![OsString::from("test"), OsString::from("--workspace")]
    } else {
        child
    };

    if arguments
        .first()
        .is_some_and(|value| value == OsStr::new("cargo"))
    {
        return Err(CoverageCommandError::CargoArgumentAfterSeparator);
    }

    let (toolchain, command_index) = if arguments
        .first()
        .is_some_and(|value| value.to_string_lossy().starts_with('+'))
    {
        (arguments.first().cloned(), 1)
    } else {
        (None, 0)
    };

    let command = arguments.get(command_index).ok_or_else(|| {
        CoverageCommandError::UnsupportedChildCommand {
            command: arguments.clone(),
        }
    })?;

    let kind = if command == OsStr::new("test") {
        TestCommandKind::CargoTest
    } else if command == OsStr::new("nextest")
        && arguments
            .get(command_index.saturating_add(1))
            .is_some_and(|value| value == OsStr::new("run"))
    {
        TestCommandKind::NextestRun
    } else {
        return Err(CoverageCommandError::UnsupportedChildCommand { command: arguments });
    };

    let manifest_path = manifest_path(&arguments)?;

    if matches!(kind, TestCommandKind::NextestRun) {
        let retry_arguments = arguments
            .get(command_index.saturating_add(2)..)
            .unwrap_or_default();
        validate_nextest_retries(retry_arguments)?;
    }

    // Commit the validated invocation without changing forwarded arguments.
    Ok(CargoInvocation {
        cargo,
        toolchain,
        arguments,
        kind,
        manifest_path,
    })
}

/// Finds one Cargo manifest-path argument without modifying forwarded arguments.
fn manifest_path(arguments: &[OsString]) -> Result<Option<PathBuf>, CoverageCommandError> {
    let mut result = None;
    let mut values = arguments.iter();

    while let Some(argument) = values.next() {
        if argument == OsStr::new("--manifest-path") {
            let value = values
                .next()
                .ok_or(CoverageCommandError::MissingManifestPathValue)?;

            result = Some(PathBuf::from(value));
        } else if let Some(value) = argument
            .to_str()
            .and_then(|value| value.strip_prefix("--manifest-path="))
        {
            result = Some(PathBuf::from(value));
        }
    }

    Ok(result)
}

/// Rejects retry settings that would create indistinguishable duplicate artifacts.
fn validate_nextest_retries(arguments: &[OsString]) -> Result<(), CoverageCommandError> {
    let mut values = arguments.iter();

    while let Some(argument) = values.next() {
        if argument == OsStr::new("--") {
            break;
        }

        let value = if argument == OsStr::new("--retries") {
            values
                .next()
                .ok_or_else(|| CoverageCommandError::UnsupportedNextestRetries {
                    value: OsString::from("missing"),
                })?
        } else if let Some(value) = argument
            .to_str()
            .and_then(|value| value.strip_prefix("--retries="))
        {
            OsStr::new(value)
        } else {
            continue;
        };

        if value != OsStr::new("0") {
            return Err(CoverageCommandError::UnsupportedNextestRetries {
                value: value.to_os_string(),
            });
        }
    }

    Ok(())
}

/// Resolves the Cargo executable without requiring UTF-8 environment data.
#[must_use]
pub fn cargo_executable() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{TestCommandKind, parse};
    #[test]
    fn accepts_test() {
        let invocation = parse(vec![OsString::from("test")], OsString::from("cargo"));

        assert!(matches!(
            invocation,
            Ok(value) if matches!(value.kind, TestCommandKind::CargoTest)
        ));
    }

    #[test]
    fn accepts_nextest_run() {
        let invocation = parse(
            ["nextest", "run"].map(OsString::from).to_vec(),
            OsString::from("cargo"),
        );

        assert!(matches!(
            invocation,
            Ok(value) if matches!(value.kind, TestCommandKind::NextestRun)
        ));
    }

    #[test]
    fn rejects_leading_cargo() {
        let invocation = parse(
            ["cargo", "test"].map(OsString::from).to_vec(),
            OsString::from("cargo"),
        );

        assert!(invocation.is_err());
    }

    #[test]
    fn rejects_nextest_retries() {
        let invocation = parse(
            ["nextest", "run", "--retries", "1"]
                .map(OsString::from)
                .to_vec(),
            OsString::from("cargo"),
        );

        assert!(invocation.is_err());
    }
}
