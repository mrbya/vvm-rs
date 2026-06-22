use std::path::{Path, PathBuf};

/// Emits Cargo rebuild dependencies for configured DUT inputs.
pub fn emit_rerun_directives(
    sources: &[PathBuf],
    hdl_include_dirs: &[PathBuf],
    bridge: &Path,
    cpp_sources: &[PathBuf],
    cpp_include_dirs: &[PathBuf],
) {
    for path in sources {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    for path in hdl_include_dirs {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    println!("cargo::rerun-if-changed={}", bridge.display());

    for path in cpp_sources {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    for path in cpp_include_dirs {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    println!("cargo::rerun-if-env-changed=VERILATOR");
    println!("cargo::rerun-if-env-changed=VERILATOR_ROOT");
}
