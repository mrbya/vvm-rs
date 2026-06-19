//! Build script for the Milestone 0 counter example.

use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    for path in [
        "build.rs",
        "cpp/counter.cpp",
        "cpp/counter.hpp",
        "rtl/counter.sv",
        "src/bridge.rs",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    let lint_status = Command::new("verilator")
        .arg("--lint-only")
        .arg("rtl/counter.sv")
        .status()?;

    if !lint_status.success() {
        return Err("Verilator failed to lint examples/counter/rtl/counter.sv".into());
    }

    cxx_build::bridge("src/bridge.rs")
        .file("cpp/counter.cpp")
        .include("cpp")
        .std("c++17")
        .compile("vvm-example-counter");

    Ok(())
}
