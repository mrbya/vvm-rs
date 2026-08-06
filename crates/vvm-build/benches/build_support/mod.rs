//! Support for subprocess-oriented `vvm-build` benchmarks.

use std::fs;
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

/// One prepared isolated consumer fixture workspace.
pub struct PreparedFixture {
    /// Owning temporary directory.
    _root: TempDir,
    /// Fixture manifest path.
    manifest_path: PathBuf,
    /// Isolated Cargo target directory.
    target_dir: PathBuf,
}

impl PreparedFixture {
    /// Creates one isolated copy of the build-consumer fixture with placeholders replaced.
    pub fn build_consumer() -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_fixture("build-consumer", "build-consumer")
    }

    /// Creates one isolated copy of an arbitrary committed fixture with placeholders replaced.
    pub fn from_fixture(
        fixture_name: &str,
        destination_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = workspace_root()?.join("tests/fixtures").join(fixture_name);
        let root = tempfile::tempdir()?;
        let consumer_root = root.path().join(destination_name);

        copy_fixture_tree(&fixture, &consumer_root)?;
        configure_build_dependencies(&consumer_root)?;

        Ok(Self {
            manifest_path: consumer_root.join("Cargo.toml"),
            target_dir: root.path().join("target"),
            _root: root,
        })
    }

    /// Appends one line to a fixture source file.
    pub fn append_line(
        &self,
        relative_path: &str,
        line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = self
            .manifest_path
            .parent()
            .ok_or("fixture manifest has no parent")?
            .join(relative_path);
        let mut contents = fs::read_to_string(&path)?;
        contents.push_str(line);
        contents.push('\n');
        fs::write(path, contents)?;
        Ok(())
    }

    /// Runs one isolated Cargo test build without executing tests.
    pub fn cargo_test_no_run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--no-run")
            .arg("--manifest-path")
            .arg(&self.manifest_path)
            .env("CARGO_TARGET_DIR", &self.target_dir)
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!(
            "cargo test --no-run failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

/// Returns the workspace root for subprocess fixtures.
pub fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other("vvm-build must be nested below the workspace root")
        })?;

    Ok(root)
}

/// Copies one fixture tree recursively.
fn copy_fixture_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_fixture_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

/// Replaces fixture dependency placeholders with the live workspace paths.
fn configure_build_dependencies(consumer_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let facade = workspace_root()?.join("crates/vvm");
    let builder = workspace_root()?.join("crates/vvm-build");
    let manifest = manifest.replace("__VVM_PATH__", &facade.to_string_lossy());
    let manifest = manifest.replace("__VVM_BUILD_PATH__", &builder.to_string_lossy());

    fs::write(manifest_path, manifest)?;

    Ok(())
}
