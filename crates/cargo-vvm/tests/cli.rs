//! Consumer-visible command-line integration coverage.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn help_describes_the_coverage_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("cargo-vvm")?;

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("coverage"));

    Ok(())
}

#[test]
fn missing_coverage_command_returns_a_usage_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::cargo_bin("cargo-vvm")?;

    command
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Usage:"));

    Ok(())
}

#[test]
fn coverage_command_writes_merged_and_rendered_counter_reports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("coverage");
    let workspace = workspace_root()?;
    let mut command = Command::cargo_bin("cargo-vvm")?;

    command
        .current_dir(workspace)
        .args(["coverage", "--output"])
        .arg(&output)
        .args([
            "--name",
            "counter",
            "--",
            "test",
            "-p",
            "vvm-example-counter",
            "counter_smoke",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("VVM functional coverage:"));

    assert!(output.join("counter.vvmcov-merged.json").is_file());
    assert!(output.join("counter.vvmcov.txt").is_file());
    assert!(output.join("counter.vvmcov.html").is_file());
    assert!(output.join("artifacts").read_dir()?.next().is_some());

    Ok(())
}

#[test]
fn coverage_command_preserves_test_failure_after_writing_counter_reports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("coverage");
    let workspace = workspace_root()?;
    let mut command = Command::cargo_bin("cargo-vvm")?;

    command
        .current_dir(workspace)
        .args(["coverage", "--output"])
        .arg(&output)
        .args([
            "--name",
            "counter",
            "--merge-policy",
            "passed-and-failed",
            "--",
            "test",
            "-p",
            "vvm-example-counter",
            "counter_fail",
            "--",
            "--ignored",
        ])
        .assert()
        .failure()
        .code(101);

    assert!(output.join("counter.vvmcov-merged.json").is_file());
    assert!(output.join("counter.vvmcov.txt").is_file());
    assert!(output.join("counter.vvmcov.html").is_file());
    assert!(output.join("artifacts").read_dir()?.next().is_some());

    Ok(())
}

fn workspace_root() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other("cargo-vvm must be nested below the workspace root")
        })?;

    Ok(root)
}
