//! Support for subprocess-oriented `cargo-vvm` benchmarks.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use criterion::BenchmarkGroup;
use criterion::measurement::WallTime;
use tempfile::TempDir;

/// Applies the expensive-workload Criterion configuration.
pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(250));
    group.measurement_time(Duration::from_secs(8));
}

/// Prepared invocation environment for the built cargo-vvm binary.
pub struct PreparedCommand {
    /// Temporary root that owns the output and target directories.
    _root: TempDir,
    /// Built cargo-vvm binary path.
    binary: PathBuf,
    /// Isolated Cargo target directory.
    target_dir: PathBuf,
    /// Isolated output directory.
    output_dir: PathBuf,
}

impl PreparedCommand {
    /// Builds the cargo-vvm binary outside timed regions and returns its path.
    pub fn build_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let workspace = workspace_root()?;
        let build = Command::new("cargo")
            .args([
                "build",
                "-p",
                "cargo-vvm",
                "--bin",
                "cargo-vvm",
                "--release",
            ])
            .current_dir(&workspace)
            .output()?;

        if !build.status.success() {
            return Err(format!(
                "cargo build -p cargo-vvm --release failed:\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            )
            .into());
        }

        Ok(workspace.join("target/release/cargo-vvm"))
    }

    /// Creates one isolated command environment around a prebuilt cargo-vvm binary.
    pub fn new(binary: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;

        Ok(Self {
            binary,
            target_dir: root.path().join("target"),
            output_dir: root.path().join("coverage"),
            _root: root,
        })
    }

    /// Runs `cargo-vvm --help`.
    pub fn help(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new(&self.binary).arg("--help").output()?;

        if output.status.success() {
            return Ok(());
        }

        Err("cargo-vvm --help failed".into())
    }

    /// Runs the representative counter coverage orchestration flow.
    pub fn counter_coverage(&self) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = workspace_root()?;
        let output = Command::new(&self.binary)
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", &self.target_dir)
            .args(["coverage", "--output"])
            .arg(&self.output_dir)
            .args([
                "--name",
                "counter",
                "--",
                "test",
                "-p",
                "vvm-example-counter",
                "counter_smoke",
            ])
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!(
            "cargo-vvm coverage failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

/// Returns the workspace root for subprocess benchmarks.
fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other("cargo-vvm must be nested below the workspace root")
        })?;

    Ok(root)
}
