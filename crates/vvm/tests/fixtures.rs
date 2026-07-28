//! Clean consumer-fixture integration coverage for the public facade.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn pure_consumer_uses_only_the_facade_from_an_isolated_workspace()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures/pure-consumer");
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join("pure-consumer");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_facade_dependency(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    assert!(
        output.status.success(),
        "pure consumer fixture failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        target_dir.is_dir(),
        "fixture did not use its isolated target directory"
    );

    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other("vvm crate must be nested below the workspace root")
        })?;

    Ok(root)
}

fn copy_fixture_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_fixture_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn configure_facade_dependency(consumer_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let facade_path = workspace_root()?.join("crates/vvm");
    let facade_path = facade_path.to_string_lossy();
    let configured_manifest = manifest.replace("__VVM_PATH__", &facade_path);

    fs::write(manifest_path, configured_manifest)?;

    Ok(())
}

#[test]
fn fixture_manifest_explicitly_opt_out_of_the_workspace() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_path = workspace_root()?.join("tests/fixtures/pure-consumer/Cargo.toml");
    let manifest = fs::read_to_string(manifest_path)?;

    assert!(manifest.contains("[workspace]"));
    assert!(!manifest.contains("vvm-core"));

    Ok(())
}

#[test]
fn packaged_consumer_builds_from_extracted_crate_archives() -> Result<(), Box<dyn std::error::Error>>
{
    let temporary_workspace = tempfile::tempdir()?;
    let package_target = temporary_workspace.path().join("package-target");
    let packages = temporary_workspace.path().join("packages");

    let core = package_and_extract("vvm-core", &package_target, &packages)?;
    let macros = package_and_extract("vvm-macros", &package_target, &packages)?;
    let facade = package_and_extract("vvm-rs", &package_target, &packages)?;

    let fixture = workspace_root()?.join("tests/fixtures/packaged-consumer");
    let consumer_root = temporary_workspace.path().join("packaged-consumer");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_packaged_dependencies(&consumer_root, &facade, &core, &macros)?;

    let target_dir = temporary_workspace.path().join("consumer-target");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    assert_command_success("packaged consumer fixture", &output)?;
    assert!(
        target_dir.is_dir(),
        "packaged fixture did not isolate its target directory"
    );

    Ok(())
}

#[test]
fn clean_consumer_generates_and_executes_a_dut() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures/build-consumer");
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join("build-consumer");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_build_dependencies(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let output = Command::new("cargo")
        .arg("test")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    assert_command_success("generated DUT consumer fixture", &output)?;
    assert!(
        target_dir.is_dir(),
        "generated DUT fixture did not isolate its target directory"
    );

    Ok(())
}

fn package_and_extract(
    package: &str,
    target_dir: &Path,
    destination: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    let output = Command::new("cargo")
        .args(["package", "--allow-dirty", "--no-verify", "-p", package])
        .env("CARGO_TARGET_DIR", target_dir)
        .output()?;

    assert_command_success("cargo package", &output)?;

    let archive_name = format!("{package}-{}.crate", env!("CARGO_PKG_VERSION"));
    let archive = target_dir.join("package").join(&archive_name);
    let extraction = Command::new("tar")
        .args(["-xzf", archive.to_string_lossy().as_ref(), "-C"])
        .arg(destination)
        .output()?;

    assert_command_success("crate archive extraction", &extraction)?;

    let directory_name = archive_name.strip_suffix(".crate").unwrap_or(&archive_name);

    Ok(destination.join(directory_name))
}

fn configure_packaged_dependencies(
    consumer_root: &Path,
    facade: &Path,
    core: &Path,
    macros: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let manifest = replace_path_placeholder(&manifest, "__VVM_PACKAGE__", facade);
    let manifest = replace_path_placeholder(&manifest, "__VVM_CORE_PACKAGE__", core);
    let manifest = replace_path_placeholder(&manifest, "__VVM_MACROS_PACKAGE__", macros);

    fs::write(manifest_path, manifest)?;

    Ok(())
}

fn configure_build_dependencies(consumer_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let facade = workspace_root()?.join("crates/vvm");
    let builder = workspace_root()?.join("crates/vvm-build");
    let manifest = replace_path_placeholder(&manifest, "__VVM_PATH__", &facade);
    let manifest = replace_path_placeholder(&manifest, "__VVM_BUILD_PATH__", &builder);

    fs::write(manifest_path, manifest)?;

    Ok(())
}

fn replace_path_placeholder(manifest: &str, placeholder: &str, path: &Path) -> String {
    manifest.replace(placeholder, &path.to_string_lossy())
}

fn assert_command_success(
    description: &str,
    output: &std::process::Output,
) -> Result<(), Box<dyn std::error::Error>> {
    if output.status.success() {
        return Ok(());
    }

    let message = format!(
        "{description} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    Err(std::io::Error::other(message).into())
}
