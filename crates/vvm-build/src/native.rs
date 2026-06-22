use std::path::{Path, PathBuf};

use crate::{BuildError, BuildResult};

/// Compiles the generated CXX bridge, generated and configured adapter
/// sources, Verilated model, and Verilator runtime.
pub fn compile(
    name: &str,
    bridge: &Path,
    cpp_sources: &[PathBuf],
    cpp_include_dirs: &[PathBuf],
    verilated_dir: &Path,
    verilator_root: &Path,
    generated_sources: &[PathBuf],
) -> BuildResult<()> {
    let verilator_include = verilator_root.join("include");
    let runtime_source = verilator_include.join("verilated.cpp");

    if !runtime_source.exists() {
        return Err(BuildError::MissingRuntimeSource {
            path: runtime_source,
        });
    }

    let mut build = cxx_build::bridge(bridge);

    build
        .include(verilated_dir)
        .include(&verilator_include)
        .include(verilator_include.join("vltstd"))
        .std("c++17");

    for include_dir in cpp_include_dirs {
        build.include(include_dir);
    }

    for source in cpp_sources {
        build.file(source);
    }

    for source in generated_sources {
        build.file(source);
    }

    build.file(runtime_source);

    let thread_runtime = verilator_include.join("verilated_threads.cpp");

    if thread_runtime.exists() {
        build.file(thread_runtime);
    }

    #[cfg(unix)]
    {
        build.flag_if_supported("-pthread");
        println!("cargo::rustc-link-lib=pthread");
    }

    build.compile(&format!("vvm_{name}"));

    Ok(())
}
