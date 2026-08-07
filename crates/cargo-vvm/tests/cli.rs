//! Consumer-visible command-line integration coverage.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
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
    let target_dir = directory.path().join("target");
    let workspace = workspace_root()?;
    let mut command = Command::cargo_bin("cargo-vvm")?;

    let output_assert = command
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
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
        .success();

    let stdout = String::from_utf8(output_assert.get_output().stdout.clone())?;
    let merged_path = output.join("counter.vvmcov-merged.json");
    let text_path = output.join("counter.vvmcov.txt");
    let html_path = output.join("counter.vvmcov.html");

    assert!(merged_path.is_file());
    assert!(text_path.is_file());
    assert!(html_path.is_file());

    let artifacts = artifact_file_names(&output)?;

    assert!(!artifacts.is_empty());
    assert!(artifacts.iter().all(|name| name.ends_with(".vvmcov.json")));

    let merged = read_json(&merged_path)?;
    let expected_metric = report_metric_line(&text_path)?;

    assert_eq!(stdout.lines().last(), Some(expected_metric.as_str()));
    assert_eq!(
        json_string(&merged, "/format")?,
        "vvm-functional-coverage-merge"
    );
    assert_eq!(json_u64(&merged, "/schema_version")?, 1);
    assert_eq!(json_string(&merged, "/producer/name")?, "vvm-rs");
    assert_eq!(json_string(&merged, "/policy")?, "passed_only");
    assert!(!json_array(&merged, "/inputs")?.is_empty());
    assert!(!json_array(&merged, "/groups")?.is_empty());

    let text = fs::read_to_string(text_path)?;
    let html = fs::read_to_string(html_path)?;

    assert!(text.contains("counter"));
    assert!(text.contains(expected_metric.as_str()));
    assert!(html.contains("<html"));
    assert!(html.contains("counter"));

    Ok(())
}

#[test]
fn coverage_command_preserves_test_failure_after_writing_counter_reports()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("coverage");
    let target_dir = directory.path().join("target");
    let workspace = workspace_root()?;
    let mut command = Command::cargo_bin("cargo-vvm")?;

    let output_assert = command
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
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

    let stdout = String::from_utf8(output_assert.get_output().stdout.clone())?;
    let merged_path = output.join("counter.vvmcov-merged.json");

    assert!(merged_path.is_file());
    assert!(output.join("counter.vvmcov.txt").is_file());
    assert!(output.join("counter.vvmcov.html").is_file());
    assert!(!artifact_file_names(&output)?.is_empty());

    let merged = read_json(&merged_path)?;
    let expected_metric = report_metric_line(&output.join("counter.vvmcov.txt"))?;

    assert_eq!(stdout.lines().last(), Some(expected_metric.as_str()));
    assert_eq!(json_string(&merged, "/policy")?, "passed_and_failed");
    assert!(json_array(&merged, "/inputs")?.iter().any(|input| {
        let status = input.pointer("/test/status").and_then(Value::as_str);
        let included = input.pointer("/included").and_then(Value::as_bool);

        status == Some("failed") && included == Some(true)
    }));

    Ok(())
}

fn artifact_file_names(
    output: &std::path::Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = fs::read_dir(output.join("artifacts"))?
        .map(|entry| {
            let entry = entry?;

            Ok::<String, std::io::Error>(entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;

    files.sort_unstable();

    Ok(files)
}

fn read_json(path: &std::path::Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn report_metric_line(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let report = fs::read_to_string(path)?;

    report
        .lines()
        .last()
        .map(str::to_owned)
        .ok_or_else(|| std::io::Error::other("coverage report was empty").into())
}

fn json_array<'a>(
    value: &'a Value,
    pointer: &str,
) -> Result<&'a [Value], Box<dyn std::error::Error>> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing JSON array at {pointer}").into())
}

fn json_string<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing JSON string at {pointer}").into())
}

fn json_u64(value: &Value, pointer: &str) -> Result<u64, Box<dyn std::error::Error>> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing JSON integer at {pointer}").into())
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
