//! Workspace integration checks for the Milestone 0 counter example.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn counter_example_assets_exist() {
    let root = repo_root();

    for relative_path in [
        "examples/counter/Cargo.toml",
        "examples/counter/build.rs",
        "examples/counter/rtl/counter.sv",
        "examples/counter/src/bridge.rs",
        "examples/counter/src/main.rs",
    ] {
        assert!(
            root.join(relative_path).is_file(),
            "missing {relative_path}"
        );
    }
}

#[test]
fn counter_example_rtl_lints_with_verilator() {
    let root = repo_root();
    let rtl_path = root.join("examples/counter/rtl/counter.sv");

    let status = Command::new("verilator")
        .arg("--lint-only")
        .arg(&rtl_path)
        .status()
        .expect("failed to run verilator");

    assert!(status.success(), "verilator lint failed for {rtl_path:?}");
}
