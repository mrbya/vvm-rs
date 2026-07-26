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
