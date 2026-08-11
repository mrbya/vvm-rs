//! Clean consumer-fixture integration coverage for the public facade.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PUBLISHABLE_PACKAGES: [&str; 5] =
    ["vvm-core", "vvm-macros", "vvm-build", "vvm-rs", "cargo-vvm"];
const INTERNAL_PATCHES: [(&str, &str); 5] = [
    ("vvm-core", "crates/vvm-core"),
    ("vvm-macros", "crates/vvm-macros"),
    ("vvm-build", "crates/vvm-build"),
    ("vvm-rs", "crates/vvm"),
    ("cargo-vvm", "crates/cargo-vvm"),
];

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
fn packaged_generated_counter_timing_and_multi_clock_fixtures_build_from_extracted_crates()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary_workspace = tempfile::tempdir()?;
    let package_target = temporary_workspace.path().join("package-target");
    let packages = temporary_workspace.path().join("packages");

    let core = package_and_extract("vvm-core", &package_target, &packages)?;
    let macros = package_and_extract("vvm-macros", &package_target, &packages)?;
    let builder = package_and_extract("vvm-build", &package_target, &packages)?;
    let facade = package_and_extract("vvm-rs", &package_target, &packages)?;

    run_packaged_build_fixture(
        "build-consumer",
        temporary_workspace.path(),
        &facade,
        &builder,
        &core,
        &macros,
    )?;
    run_packaged_build_fixture(
        "native-timing-delay",
        temporary_workspace.path(),
        &facade,
        &builder,
        &core,
        &macros,
    )?;
    run_packaged_build_fixture(
        "native-multi-clock-counter",
        temporary_workspace.path(),
        &facade,
        &builder,
        &core,
        &macros,
    )?;

    Ok(())
}

#[test]
fn packaged_coverage_fixture_and_installed_cargo_vvm_workflow_succeed()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary_workspace = tempfile::tempdir()?;
    let package_target = temporary_workspace.path().join("package-target");
    let packages = temporary_workspace.path().join("packages");

    let core = package_and_extract("vvm-core", &package_target, &packages)?;
    let macros = package_and_extract("vvm-macros", &package_target, &packages)?;
    let facade = package_and_extract("vvm-rs", &package_target, &packages)?;
    let cargo_vvm = package_and_extract("cargo-vvm", &package_target, &packages)?;
    let extracted = vec![
        ("vvm-core".to_owned(), core.clone()),
        ("vvm-macros".to_owned(), macros.clone()),
        ("vvm-rs".to_owned(), facade.clone()),
        ("cargo-vvm".to_owned(), cargo_vvm.clone()),
    ];

    let fixture = workspace_root()?.join("tests/fixtures/coverage-consumer");
    let consumer_root = temporary_workspace.path().join("coverage-consumer");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_packaged_facade_fixture(&consumer_root, &facade, &core, &macros)?;

    let cargo_home = temporary_workspace.path().join("cargo-home");
    let install_root = temporary_workspace.path().join("cargo-install");
    let install_target = temporary_workspace.path().join("install-target");
    let install_output = Command::new("cargo")
        .arg("install")
        .arg("--locked")
        .arg("--path")
        .arg(&cargo_vvm)
        .arg("--root")
        .arg(&install_root)
        .args(extracted_patch_arguments(&extracted)?)
        .env("CARGO_HOME", &cargo_home)
        .env("CARGO_TARGET_DIR", &install_target)
        .output()?;

    assert_command_success("packaged cargo-vvm install", &install_output)?;

    let cargo_vvm_binary = install_root.join("bin/cargo-vvm");
    assert!(
        cargo_vvm_binary.is_file(),
        "packaged cargo-vvm install did not create an isolated binary"
    );

    let help_output = Command::new(&cargo_vvm_binary)
        .arg("--help")
        .env("CARGO_HOME", &cargo_home)
        .output()?;
    assert_command_success("installed cargo-vvm --help", &help_output)?;

    let coverage_output = temporary_workspace.path().join("coverage-output");
    let consumer_target = temporary_workspace.path().join("coverage-target");
    let workflow_output = Command::new(&cargo_vvm_binary)
        .current_dir(&consumer_root)
        .env("CARGO_HOME", &cargo_home)
        .env("CARGO_TARGET_DIR", &consumer_target)
        .arg("coverage")
        .arg("--output")
        .arg(&coverage_output)
        .args([
            "--name",
            "fixture",
            "--",
            "test",
            "packaged_manual_result_with_coverage",
        ])
        .output()?;

    assert_command_success("installed cargo-vvm coverage workflow", &workflow_output)?;
    assert!(
        String::from_utf8_lossy(&workflow_output.stdout).contains("VVM functional coverage:"),
        "installed cargo-vvm coverage workflow did not print the coverage summary line"
    );
    assert!(
        coverage_output.join("fixture.vvmcov-merged.json").is_file(),
        "installed cargo-vvm coverage workflow did not write merged JSON"
    );
    assert!(
        coverage_output.join("fixture.vvmcov.txt").is_file(),
        "installed cargo-vvm coverage workflow did not write the text report"
    );
    assert!(
        coverage_output.join("fixture.vvmcov.html").is_file(),
        "installed cargo-vvm coverage workflow did not write the HTML report"
    );
    assert!(
        coverage_output
            .join("artifacts")
            .read_dir()?
            .next()
            .is_some(),
        "installed cargo-vvm coverage workflow did not retain per-test artifacts"
    );
    assert!(
        consumer_target.is_dir(),
        "installed cargo-vvm coverage workflow did not isolate its target directory"
    );

    Ok(())
}

#[test]
fn packaged_publishable_crates_resolve_from_extracted_archives()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary_workspace = tempfile::tempdir()?;
    let package_target = temporary_workspace.path().join("package-target");
    let packages = temporary_workspace.path().join("packages");

    let extracted = PUBLISHABLE_PACKAGES
        .into_iter()
        .map(|package| {
            let extracted_path = package_and_extract(package, &package_target, &packages)?;

            assert_packaged_manifest_is_self_contained(package, &extracted_path)?;

            Ok::<(String, PathBuf), Box<dyn std::error::Error>>((
                package.to_owned(),
                extracted_path,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for extracted_package in &extracted {
        let package = &extracted_package.0;
        let package_root = &extracted_package.1;

        let target_dir = temporary_workspace
            .path()
            .join(format!("publication-target-{package}"));
        let output = Command::new("cargo")
            .arg("check")
            .arg("--offline")
            .arg("--manifest-path")
            .arg(package_root.join("Cargo.toml"))
            .args(extracted_patch_arguments(&extracted)?)
            .env("CARGO_TARGET_DIR", &target_dir)
            .output()?;

        assert_command_success(
            &format!("packaged publication check for {package}"),
            &output,
        )?;
        assert!(
            target_dir.is_dir(),
            "packaged publication check for {package} did not isolate its target directory"
        );
    }

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

#[test]
fn documentation_quick_start_fixture_builds_and_runs() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures/docs-quick-start");
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join("docs-quick-start");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_build_dependencies(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let trace_dir = temporary_workspace.path().join("trace");
    let output = Command::new("cargo")
        .arg("test")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("VVM_TRACE_DIR", &trace_dir)
        .output()?;

    assert_command_success("documentation quick-start fixture", &output)?;
    assert!(
        target_dir.is_dir(),
        "quick-start fixture did not isolate its target directory"
    );

    Ok(())
}

#[test]
fn documentation_inout_fixture_builds_and_runs() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures/docs-inout-line");
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join("docs-inout-line");

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_build_dependencies(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let output = Command::new("cargo")
        .arg("test")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    assert_command_success("documentation inout fixture", &output)?;
    assert!(
        target_dir.is_dir(),
        "inout fixture did not isolate its target directory"
    );

    Ok(())
}

#[test]
fn native_packed_array_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-packed-array")
}

#[test]
fn native_packed_enum_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-packed-enum")
}

#[test]
fn native_packed_struct_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-packed-struct")
}

#[test]
fn native_multi_clock_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-multi-clock-counter")
}

#[test]
fn native_signed_adder_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-signed-adder")
}

#[test]
fn native_timing_delay_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-timing-delay")
}

#[test]
fn native_thread_confinement_fixture_rejects_send_and_sync()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_compile_fail_fixture(
        "native-thread-confinement",
        &[
            "cannot be sent between threads safely",
            "cannot be shared between threads safely",
        ],
    )
}

#[test]
fn native_unpacked_array_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-unpacked-array")
}

#[test]
fn native_wide_transform_fixture_preserves_generated_port_regression()
-> Result<(), Box<dyn std::error::Error>> {
    run_native_fixture("native-wide-transform")
}

fn run_native_fixture(fixture_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures").join(fixture_name);
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join(fixture_name);

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_build_dependencies(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let trace_dir = temporary_workspace.path().join("trace");
    let coverage_dir = temporary_workspace.path().join("coverage");
    let output = Command::new("cargo")
        .arg("test")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("VVM_TRACE_DIR", &trace_dir)
        .env("VVM_COVERAGE_DIR", &coverage_dir)
        .output()?;

    assert_command_success(fixture_name, &output)
}

fn run_native_compile_fail_fixture(
    fixture_name: &str,
    expected_fragments: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures").join(fixture_name);
    let temporary_workspace = tempfile::tempdir()?;
    let consumer_root = temporary_workspace.path().join(fixture_name);

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_build_dependencies(&consumer_root)?;

    let target_dir = temporary_workspace.path().join("target");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--tests")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;

    if output.status.success() {
        let message = format!(
            "{fixture_name} unexpectedly compiled\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        return Err(std::io::Error::other(message).into());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    for fragment in expected_fragments {
        if !stderr.contains(fragment) {
            let message = format!(
                "{fixture_name} failed for the wrong reason; missing diagnostic fragment \
                 `{fragment}`\nstderr:\n{stderr}"
            );

            return Err(std::io::Error::other(message).into());
        }
    }

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
        .args(internal_patch_arguments()?)
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

fn internal_patch_arguments() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let root = workspace_root()?;

    let arguments = INTERNAL_PATCHES
        .iter()
        .flat_map(|entry| {
            let name = &entry.0;
            let path = &entry.1;
            let absolute_path = root.join(path);

            [
                "--config".to_owned(),
                format!(
                    "patch.crates-io.{name}.path=\"{}\"",
                    absolute_path.to_string_lossy()
                ),
            ]
        })
        .collect::<Vec<_>>();

    Ok(arguments)
}

fn assert_packaged_manifest_is_self_contained(
    package: &str,
    package_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = fs::read_to_string(package_root.join("Cargo.toml"))?;

    assert!(
        !manifest.contains("workspace = true"),
        "packaged manifest for {package} still inherits workspace metadata:\n{manifest}"
    );
    assert!(
        !manifest.contains("path = \"/home/"),
        "packaged manifest for {package} leaked an absolute workspace path:\n{manifest}"
    );
    assert!(
        package_root.join("LICENSE-MIT").is_file(),
        "packaged archive for {package} is missing LICENSE-MIT"
    );
    assert!(
        package_root.join("LICENSE-APACHE").is_file(),
        "packaged archive for {package} is missing LICENSE-APACHE"
    );

    Ok(())
}

fn extracted_patch_arguments(
    packages: &[(String, PathBuf)],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let patches = packages
        .iter()
        .map(|entry| {
            let name = &entry.0;
            let path = &entry.1;

            Ok::<Vec<String>, std::io::Error>(vec![
                "--config".to_owned(),
                format!("patch.crates-io.{name}.path=\"{}\"", path.to_string_lossy()),
            ])
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    Ok(patches)
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

fn configure_packaged_facade_fixture(
    consumer_root: &Path,
    facade: &Path,
    core: &Path,
    macros: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let manifest = replace_path_placeholder(&manifest, "__VVM_PACKAGE__", facade);
    let manifest = append_packaged_patches(&manifest, core, macros);

    fs::write(manifest_path, manifest)?;

    Ok(())
}

fn run_packaged_build_fixture(
    fixture_name: &str,
    temporary_root: &Path,
    facade: &Path,
    builder: &Path,
    core: &Path,
    macros: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = workspace_root()?.join("tests/fixtures").join(fixture_name);
    let consumer_root = temporary_root.join(format!("packaged-{fixture_name}"));

    copy_fixture_tree(&fixture, &consumer_root)?;
    configure_packaged_build_fixture(&consumer_root, facade, builder, core, macros)?;

    let target_dir = temporary_root.join(format!("target-{fixture_name}"));
    let trace_dir = temporary_root.join(format!("trace-{fixture_name}"));
    let coverage_dir = temporary_root.join(format!("coverage-{fixture_name}"));
    let output = Command::new("cargo")
        .arg("test")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(consumer_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("VVM_TRACE_DIR", &trace_dir)
        .env("VVM_COVERAGE_DIR", &coverage_dir)
        .output()?;

    assert_command_success(fixture_name, &output)?;
    assert!(
        target_dir.is_dir(),
        "packaged fixture {fixture_name} did not isolate its target directory"
    );

    Ok(())
}

fn configure_packaged_build_fixture(
    consumer_root: &Path,
    facade: &Path,
    builder: &Path,
    core: &Path,
    macros: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = consumer_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    let manifest = replace_path_placeholder(&manifest, "__VVM_PATH__", facade);
    let manifest = replace_path_placeholder(&manifest, "__VVM_BUILD_PATH__", builder);
    let manifest = append_packaged_patches(&manifest, core, macros);

    fs::write(manifest_path, manifest)?;

    Ok(())
}

fn append_packaged_patches(manifest: &str, core: &Path, macros: &Path) -> String {
    let core_path = core.to_string_lossy();
    let macros_path = macros.to_string_lossy();

    format!(
        "{manifest}\n[patch.crates-io]\nvvm-core = {{ path = \"{core_path}\" }}\nvvm-macros = {{ \
         path = \"{macros_path}\" }}\n"
    )
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
