use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::{BuildError, BuildResult, verilator};

/// Configures generation and native compilation of a Verilated DUT.
#[derive(Debug)]
pub struct DutBuilder {
    /// Logical DUT name used for generated files and symbols.
    name: String,

    /// HDL top-module name.
    top_module: Option<String>,

    /// HDL sources.
    sources: Vec<PathBuf>,

    /// Hand-written CXX bridge source.
    bridge: Option<PathBuf>,

    /// Additional C++ translation units.
    cpp_sources: Vec<PathBuf>,

    /// Additional C++ include directories.
    cpp_include_dirs: Vec<PathBuf>,

    /// Additional raw Verilator arguments.
    verilator_arguments: Vec<OsString>,
}

impl DutBuilder {
    /// Creates a build configuration for a DUT.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            top_module: None,
            sources: Vec::new(),
            bridge: None,
            cpp_sources: Vec::new(),
            cpp_include_dirs: Vec::new(),
            verilator_arguments: Vec::new(),
        }
    }

    /// Sets the HDL top-module name.
    #[must_use]
    pub fn top_module(mut self, top_module: impl Into<String>) -> Self {
        self.top_module = Some(top_module.into());
        self
    }

    /// Adds a HDL source file.
    #[must_use]
    pub fn source(mut self, source: impl Into<PathBuf>) -> Self {
        self.sources.push(source.into());
        self
    }

    /// Sets the Rust source containing the CXX bridge declaration.
    #[must_use]
    pub fn bridge(mut self, bridge: impl Into<PathBuf>) -> Self {
        self.bridge = Some(bridge.into());
        self
    }

    /// Adds a C++ translation unit.
    #[must_use]
    pub fn cpp_source(mut self, source: impl Into<PathBuf>) -> Self {
        self.cpp_sources.push(source.into());
        self
    }

    /// Adds a C++ include directory.
    #[must_use]
    pub fn cpp_include(mut self, include_dir: impl Into<PathBuf>) -> Self {
        self.cpp_include_dirs.push(include_dir.into());
        self
    }

    /// Adds a raw argument passed to Verilator.
    #[must_use]
    pub fn verilator_arg(mut self, argument: impl Into<OsString>) -> Self {
        self.verilator_arguments.push(argument.into());
        self
    }

    /// Generates the Verilated model and compiles the native bridge.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError`] if:
    /// - the configuration is incomplete,
    /// - Verilator fails,
    /// - required files are unavailable,
    /// - filesystem access fails.
    pub fn build(self) -> BuildResult<()> {
        validate_identifier("DUT name", &self.name)?;

        let top_module = self.top_module.ok_or(BuildError::MissingTopModule)?;

        validate_identifier("top module", &top_module)?;

        if self.sources.is_empty() {
            return Err(BuildError::MissingSources);
        }

        let bridge = self.bridge.ok_or(BuildError::MissingBridge)?;

        let manifest_dir = required_environment_path("CARGO_MANIFEST_DIR")?;
        let out_dir = required_environment_path("OUT_DIR")?;

        let model_prefix = format!("V{}", self.name);
        let dut_output_dir = out_dir.join("vvm").join(&self.name);
        let verilated_dir = dut_output_dir.join("verilated");

        create_directory(&verilated_dir)?;

        let sources = resolve_paths(&manifest_dir, &self.sources);
        let bridge = resolve_path(&manifest_dir, &bridge);
        let cpp_sources = resolve_paths(&manifest_dir, &self.cpp_sources);
        let cpp_include_dirs = resolve_paths(&manifest_dir, &self.cpp_include_dirs);

        emit_rerun_directives(&sources, &bridge, &cpp_sources, &cpp_include_dirs);

        let executable = verilator::executable();

        verilator::generate(
            &executable,
            &top_module,
            &model_prefix,
            &verilated_dir,
            &sources,
            &self.verilator_arguments,
        )?;

        let verilator_root = verilator::root(&executable)?;

        let generated_sources = verilator::generated_sources(&verilated_dir, &model_prefix)?;

        compile_native_sources(
            &self.name,
            &bridge,
            &cpp_sources,
            &cpp_include_dirs,
            &verilated_dir,
            &verilator_root,
            &generated_sources,
        )
    }
}

/// Reads a required Cargo build-script environment path.
fn required_environment_path(name: &'static str) -> Result<PathBuf, BuildError> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironmentVariable { name })
}

/// Creates an output directory.
fn create_directory(path: &Path) -> Result<(), BuildError> {
    fs::create_dir_all(path).map_err(|source| BuildError::Io {
        operation: "create output directory",
        path: path.to_path_buf(),
        source,
    })
}

/// Resolves a user path relative to the consuming package.
fn resolve_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

/// Resolves multiple paths relative to the consuming package.
fn resolve_paths(manifest_dir: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| resolve_path(manifest_dir, path))
        .collect()
}

/// Emits Cargo rebuild dependencies.
fn emit_rerun_directives(
    sources: &[PathBuf],
    bridge: &Path,
    cpp_sources: &[PathBuf],
    cpp_include_dirs: &[PathBuf],
) {
    for path in sources {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    println!("cargo::rerun-if-changed={}", bridge.display());

    for path in cpp_sources {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    // Watching the include directory also tracks headers below it.
    for path in cpp_include_dirs {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    println!("cargo::rerun-if-env-changed=VERILATOR");
    println!("cargo::rerun-if-env-changed=VERILATOR_ROOT");
}

/// Compiles the CXX bridge, adapter, Verilated model, and runtime.
fn compile_native_sources(
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

    let library_name = format!("vvm_{name}");
    build.compile(&library_name);

    Ok(())
}

/// Checks whether a name can be safely used as a C++ identifier.
///
/// # Errors
/// Returns [`BuildError::InvalidIdentifier`] if:
/// - identifier starts with a non alphabetic character,
/// - identifier contains invalid characters (only aphanumeric + '_' allowed).
pub fn validate_identifier(field: &'static str, value: &str) -> BuildResult<()> {
    let mut characters = value.chars();

    let Some(first) = characters.next() else {
        return Err(BuildError::InvalidIdentifier {
            field,
            value: value.to_owned(),
        });
    };

    let valid_first = first == '_' || first.is_alphabetic();

    let valid_remaining =
        characters.all(|character| character == '_' || character.is_ascii_alphanumeric());

    if valid_first && valid_remaining {
        return Ok(());
    }

    Err(BuildError::InvalidIdentifier {
        field,
        value: value.to_owned(),
    })
}
