//! Builds the handwritten CXX bridge and Verilated counter model.

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

/// Default Verilator executable name.
const DEFAULT_VERILATOR: &str = "verilator";

/// Name of the generated Verilator top-level class.
const MODEL_PREFIX: &str = "Vcounter";

fn main() -> Result<(), Box<dyn Error>> {
    emit_rerun_directives();

    let out_dir = required_environment_path("OUT_DIR")?;
    let verilated_dir = out_dir.join("verilated");

    fs::create_dir_all(&verilated_dir)?;

    let verilator = verilator_executable();

    generate_model(&verilator, &verilated_dir)?;

    let verilator_root = query_verilator_root(&verilator)?;
    let generated_sources = find_generated_sources(&verilated_dir)?;

    compile_native_sources(&verilated_dir, &verilator_root, &generated_sources);

    Ok(())
}

/// Emits Cargo build-script dependency information.
fn emit_rerun_directives() {
    for path in [
        "build.rs",
        "cpp/counter.cpp",
        "cpp/counter.hpp",
        "rtl/counter.sv",
        "src/bridge.rs",
    ] {
        println!("cargo::rerun-if-changed={path}");
    }

    println!("cargo::rerun-if-env-changed=VERILATOR");
    println!("cargo::rerun-if-env-changed=VERILATOR_ROOT");
}

/// Returns a required environment variable as a filesystem path.
fn required_environment_path(name: &'static str) -> Result<PathBuf, io::Error> {
    env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("required environment variable `{name}` is not set"),
        )
    })
}

/// Returns the configured Verilator executable.
fn verilator_executable() -> OsString {
    env::var_os("VERILATOR").unwrap_or_else(|| OsString::from(DEFAULT_VERILATOR))
}

/// Invokes Verilator to generate the counter C++ model.
fn generate_model(verilator: &OsStr, verilated_dir: &Path) -> Result<(), io::Error> {
    let status = Command::new(verilator)
        .arg("--cc")
        .arg("--top-module")
        .arg("counter")
        .arg("--prefix")
        .arg(MODEL_PREFIX)
        .arg("--Mdir")
        .arg(verilated_dir)
        .arg("--emit-accessors")
        .arg("rtl/counter.sv")
        .status()
        .map_err(|source| {
            io::Error::new(
                source.kind(),
                format!("failed to execute Verilator: {source}"),
            )
        })?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "Verilator model generation failed with status {status}"
        )));
    }

    Ok(())
}

/// Queries the installation's Verilator root directory.
fn query_verilator_root(verilator: &OsStr) -> Result<PathBuf, Box<dyn Error>> {
    let output = Command::new(verilator)
        .arg("--getenv")
        .arg("VERILATOR_ROOT")
        .output()?;

    if !output.status.success() {
        return Err(
            io::Error::other(format!("failed to query VERILATOR_ROOT: {}", output.status)).into(),
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    let root = stdout.trim();

    if root.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Verilator returned an empty VERILATOR_ROOT",
        )
        .into());
    }

    Ok(PathBuf::from(root))
}

/// Finds Verilator-generated C++ translation units.
fn find_generated_sources(verilated_dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut sources = Vec::new();

    for entry in fs::read_dir(verilated_dir)? {
        let path = entry?.path();

        let is_cpp = path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("cpp"));

        let is_combined_source = path
            .file_name()
            .is_some_and(|name| name == OsStr::new("Vcounter__ALL.cpp"));

        if is_cpp && !is_combined_source {
            sources.push(path);
        }
    }

    sources.sort();

    if sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Verilator generated no C++ sources under `{}`",
                verilated_dir.display()
            ),
        ));
    }

    Ok(sources)
}

/// Compiles the bridge, adapter scaffold, model, and Verilator runtime.
fn compile_native_sources(
    verilated_dir: &Path,
    verilator_root: &Path,
    generated_sources: &[PathBuf],
) {
    let verilator_include = verilator_root.join("include");

    let mut build = cxx_build::bridge("src/bridge.rs");

    build
        .file("cpp/counter.cpp")
        .include("cpp")
        .include(verilated_dir)
        .include(&verilator_include)
        .include(verilator_include.join("vltstd"))
        .std("c++17");

    for source in generated_sources {
        build.file(source);
    }

    // These provide the VerilatedContext and common runtime implementation.
    for runtime_source in ["verilated.cpp", "verilated_threads.cpp"] {
        let path = verilator_include.join(runtime_source);

        if path.exists() {
            build.file(path);
        }
    }

    #[cfg(unix)]
    {
        build.flag_if_supported("-pthread");
        println!("cargo::rustc-link-lib=pthread");
    }

    build.compile("vvm-example-counter");
}
