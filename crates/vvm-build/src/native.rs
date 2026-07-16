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
    /// Whether the Verilated model uses timing constructs.
    pub timing: bool,
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
        timing,
    } = *inputs;

    let verilator_include = verilator_root.join("include");
    let runtime_sources = required_runtime_sources(&verilator_include, trace.is_some(), timing)?;

    let mut build = cxx_build::bridge(bridge);

    let verilator_vltstd = verilator_include.join("vltstd");

    build
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-parameter")
        .std(cpp_standard(timing));

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

    for source in runtime_sources {
        build.file(source);
    }

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

/// Selects the C++ standard required by a model configuration.
#[must_use]
const fn cpp_standard(timing: bool) -> &'static str {
    if timing { "c++20" } else { "c++17" }
}

/// Returns required Verilator runtime sources for the selected capabilities.
fn required_runtime_sources(
    verilator_include: &Path,
    trace: bool,
    timing: bool,
) -> BuildResult<Vec<PathBuf>> {
    let mut sources = vec![verilator_include.join("verilated.cpp")];

    if trace {
        sources.push(verilator_include.join("verilated_vcd_c.cpp"));
    }

    if timing {
        sources.push(timing_runtime_source(verilator_include));
    }

    for source in &sources {
        if !source.exists() {
            return Err(BuildError::MissingRuntimeSource {
                path: source.clone(),
            });
        }
    }

    Ok(sources)
}

/// Returns the Verilator timing runtime path.
fn timing_runtime_source(verilator_include: &Path) -> PathBuf {
    verilator_include.join("verilated_timing.cpp")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{cpp_standard, required_runtime_sources, timing_runtime_source};
    use crate::BuildError;

    #[test]
    fn ordinary_build_uses_cpp17() {
        assert_eq!(cpp_standard(false), "c++17");
    }

    #[test]
    fn timing_build_uses_cpp20() {
        assert_eq!(cpp_standard(true), "c++20");
    }

    #[test]
    fn ordinary_runtime_does_not_require_timing_source() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        fs::write(directory.path().join("verilated.cpp"), "")?;

        assert_eq!(
            required_runtime_sources(directory.path(), false, false)?.len(),
            1
        );

        Ok(())
    }

    #[test]
    fn timing_runtime_is_required() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        fs::write(directory.path().join("verilated.cpp"), "")?;

        let expected = timing_runtime_source(directory.path());

        assert!(matches!(
            required_runtime_sources(directory.path(), false, true),
            Err(BuildError::MissingRuntimeSource { path }) if path == expected
        ));

        Ok(())
    }

    #[test]
    fn timing_and_trace_select_both_runtimes() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;

        for name in [
            "verilated.cpp",
            "verilated_vcd_c.cpp",
            "verilated_timing.cpp",
        ] {
            fs::write(directory.path().join(name), "")?;
        }

        assert_eq!(
            required_runtime_sources(directory.path(), true, true)?.len(),
            3
        );

        Ok(())
    }
}
