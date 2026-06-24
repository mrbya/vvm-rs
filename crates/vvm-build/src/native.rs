use std::path::{Path, PathBuf};

use crate::{BuildError, BuildResult, TraceOptions};

/// Native compilation inputs for one generated DUT.
pub struct CompileInputs<'a> {
    /// Logical DUT name.
    pub name: &'a str,
    /// Generated CXX bridge source.
    pub bridge: &'a Path,
    /// Generated adapter and Verilated C++ sources.
    pub cpp_sources: &'a [PathBuf],
    /// C++ include directories.
    pub cpp_include_dirs: &'a [PathBuf],
    /// Directory containing Verilator generated headers.
    pub verilated_dir: &'a Path,
    /// Verilator installation root.
    pub verilator_root: &'a Path,
    /// Extra generated native sources.
    pub generated_sources: &'a [PathBuf],
    /// Optional waveform trace configuration.
    pub trace: Option<TraceOptions>,
}

/// Compiles the generated CXX bridge, generated and configured adapter
/// sources, Verilated model, and Verilator runtime.
pub fn compile(inputs: &CompileInputs<'_>) -> BuildResult<()> {
    let CompileInputs {
        name,
        bridge,
        cpp_sources,
        cpp_include_dirs,
        verilated_dir,
        verilator_root,
        generated_sources,
        trace,
    } = *inputs;

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
        .flag_if_supported("-Wno-unused-parameter")
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
