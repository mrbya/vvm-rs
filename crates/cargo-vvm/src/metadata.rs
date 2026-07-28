//! Cargo metadata lookup used to select the default output root.

use std::path::PathBuf;

use serde::Deserialize;

use crate::command::CargoInvocation;
use crate::error::CoverageCommandError;
use crate::process::{MetadataRequest, ProcessRunner};

/// Minimal stable portion of Cargo metadata needed by this command.
#[derive(Debug, Deserialize)]
struct CargoMetadata {
    /// Absolute workspace root reported by Cargo.
    workspace_root: PathBuf,

    /// Target directory reported by Cargo.
    target_directory: PathBuf,
}

/// Resolved Cargo workspace locations.
#[derive(Debug, Clone)]
pub struct WorkspaceMetadata {
    /// Cargo target directory.
    pub(crate) target_directory: PathBuf,
}

/// Reads Cargo metadata through the supplied process runner.
///
/// # Errors
///
/// Returns an error when Cargo cannot run, fails, or emits invalid JSON.
pub fn query(
    runner: &mut impl ProcessRunner,
    invocation: &CargoInvocation,
) -> Result<WorkspaceMetadata, CoverageCommandError> {
    let request = MetadataRequest {
        program: invocation.cargo.clone(),
        toolchain: invocation.toolchain.clone(),
        manifest_path: invocation.manifest_path.clone(),
        current_dir: std::env::current_dir().map_err(|source| {
            CoverageCommandError::CreateOutputDirectory {
                path: PathBuf::from("."),
                source,
            }
        })?,
    };

    let output =
        runner
            .cargo_metadata(&request)
            .map_err(|source| CoverageCommandError::MetadataSpawn {
                program: request.program.clone(),
                source,
            })?;

    if !output.status.success() {
        return Err(CoverageCommandError::MetadataFailed {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|source| CoverageCommandError::MetadataDecode { source })?;

    let CargoMetadata {
        workspace_root,
        target_directory,
    } = metadata;

    drop(workspace_root);

    // Keep only the location required to derive coverage output paths.
    Ok(WorkspaceMetadata { target_directory })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::io;
    use std::path::PathBuf;
    use std::process::{Command, ExitStatus};

    use super::query;
    use crate::command::parse;
    use crate::error::CoverageCommandError;
    use crate::process::{MetadataRequest, ProcessOutput, ProcessRunner, TestProcessRequest};

    struct FakeRunner {
        output: ProcessOutput,
        request: Option<MetadataRequest>,
    }

    impl ProcessRunner for FakeRunner {
        fn cargo_metadata(
            &mut self,
            request: &MetadataRequest,
        ) -> Result<ProcessOutput, io::Error> {
            self.request = Some(request.clone());

            Ok(ProcessOutput {
                status: self.output.status,
                stdout: self.output.stdout.clone(),
                stderr: self.output.stderr.clone(),
            })
        }

        fn run_tests(&mut self, _: &TestProcessRequest) -> Result<ExitStatus, io::Error> {
            Err(io::Error::other("metadata tests do not run children"))
        }
    }

    fn successful_status() -> Result<ExitStatus, io::Error> {
        Command::new("true").status()
    }

    #[test]
    fn forwards_cargo_selection_and_extracts_target_directory()
    -> Result<(), Box<dyn std::error::Error>> {
        let invocation = parse(
            ["+nightly", "test", "--manifest-path", "project/Cargo.toml"]
                .map(OsString::from)
                .to_vec(),
            OsString::from("selected-cargo"),
        )?;
        let mut runner = FakeRunner {
            output: ProcessOutput {
                status: successful_status()?,
                stdout: br#"{"workspace_root":"/workspace","target_directory":"/target"}"#.to_vec(),
                stderr: Vec::new(),
            },
            request: None,
        };

        let metadata = query(&mut runner, &invocation)?;
        let request = runner.request.ok_or("metadata request was not captured")?;

        assert_eq!(metadata.target_directory, PathBuf::from("/target"));
        assert_eq!(request.program, "selected-cargo");
        assert_eq!(request.toolchain, Some(OsString::from("+nightly")));
        assert_eq!(
            request.manifest_path,
            Some(PathBuf::from("project/Cargo.toml"))
        );

        Ok(())
    }

    #[test]
    fn rejects_invalid_metadata_json() -> Result<(), Box<dyn std::error::Error>> {
        let invocation = parse(
            ["test"].map(OsString::from).to_vec(),
            OsString::from("cargo"),
        )?;
        let mut runner = FakeRunner {
            output: ProcessOutput {
                status: successful_status()?,
                stdout: b"not json".to_vec(),
                stderr: Vec::new(),
            },
            request: None,
        };

        assert!(matches!(
            query(&mut runner, &invocation),
            Err(CoverageCommandError::MetadataDecode { .. })
        ));

        Ok(())
    }
}
