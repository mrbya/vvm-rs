use std::path::{Path, PathBuf};

use crate::{BuildError, BuildResult, TraceOptions};

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
    trace: Option<TraceOptions>,
) -> BuildResult<()> {
    let verilator_include = verilator_root.join("include");
    let runtime_source = verilator_include.join("verilated.cpp");

    if !runtime_source.exists() {
        return Err(BuildError::MissingRuntimeSource {
            path: runtime_source,
        });
    }

    let mut build = cxx_build::bridge(bridge);

    let verilator_vltstd = verilator_include.join("vltstd");

    build
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-variable")
        .std("c++17");

    let compiler = build.get_compiler();

    if compiler.is_like_msvc() {
        build
            .flag("/external:I")
            .flag(verilated_dir)
            .flag("/external:I")
            .flag(&verilator_include)
            .flag("/external:I")
            .flag(&verilator_vltstd)
            .flag("/external:W0");
    } else {
        build
            .flag("-isystem")
            .flag(verilated_dir)
            .flag("-isystem")
            .flag(&verilator_include)
            .flag("-isystem")
            .flag(&verilator_vltstd);
    }

    for include_dir in cpp_include_dirs {
        build.include(include_dir);
    }

    for source in cpp_sources {
        build.file(source);
    }

    for source in generated_sources {
        build.file(source);
    }

    if trace.is_some() {
        let trace_runtime = verilator_include.join("verilated_vcd_c.cpp");

        if !trace_runtime.exists() {
            return Err(BuildError::MissingRuntimeSource {
                path: trace_runtime,
            });
        }

        build.file(trace_runtime);
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
