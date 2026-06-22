use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::error::{BuildError, BuildResult};
use crate::{cargo, native, paths, verilator};

/// HDL preprocessor definition configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Define {
    /// Definition name.
    pub name: String,

    /// Optional definition value.
    pub value: Option<String>,
}

/// Configures generation and native compilation of a Verilated DUT.
#[derive(Debug)]
pub struct DutBuilder {
    /// Logical DUT name used for generated files and symbols.
    name: String,

    /// HDL top-module name.
    top_module: Option<String>,

    /// HDL sources.
    sources: Vec<PathBuf>,

    /// HDL include directories.
    hdl_include_dirs: Vec<PathBuf>,

    /// HDL preprocessor definitions.
    defines: Vec<Define>,

    /// Hand-written CXX bridge source.
    bridge: Option<PathBuf>,

    /// Additional C++ translation units.
    cpp_sources: Vec<PathBuf>,

    /// Additional C++ include directories.
    cpp_include_dirs: Vec<PathBuf>,

    /// Additional raw Verilator arguments.
    verilator_arguments: Vec<OsString>,

    /// Explicit Verilator executable override.
    verilator_executable: Option<OsString>,
}

impl DutBuilder {
    /// Creates a build configuration for a DUT.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            top_module: None,
            sources: Vec::new(),
            hdl_include_dirs: Vec::new(),
            defines: Vec::new(),
            bridge: None,
            cpp_sources: Vec::new(),
            cpp_include_dirs: Vec::new(),
            verilator_arguments: Vec::new(),
            verilator_executable: None,
        }
    }

    /// Sets the HDL top-module name.
    #[must_use]
    pub fn top_module(mut self, top_module: impl Into<String>) -> Self {
        self.top_module = Some(top_module.into());
        self
    }

    /// Adds an HDL source file.
    #[must_use]
    pub fn source(mut self, source: impl Into<PathBuf>) -> Self {
        self.sources.push(source.into());
        self
    }

    /// Adds HDL source files.
    #[must_use]
    pub fn sources<I, P>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.sources.extend(sources.into_iter().map(Into::into));
        self
    }

    /// Adds a directory searched for HDL include files.
    #[must_use]
    pub fn hdl_include(mut self, include_dir: impl Into<PathBuf>) -> Self {
        self.hdl_include_dirs.push(include_dir.into());
        self
    }

    /// Adds directories searched for HDL include files.
    #[must_use]
    pub fn hdl_includes<I, P>(mut self, include_dirs: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.hdl_include_dirs
            .extend(include_dirs.into_iter().map(Into::into));
        self
    }

    /// Defines an HDL preprocessor symbol without a value.
    #[must_use]
    pub fn define(mut self, name: impl Into<String>) -> Self {
        self.defines.push(Define {
            name: name.into(),
            value: None,
        });
        self
    }

    /// Defines an HDL preprocessor symbol with a value.
    #[must_use]
    pub fn define_value(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.defines.push(Define {
            name: name.into(),
            value: Some(value.into()),
        });
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

    /// Adds C++ translation units.
    #[must_use]
    pub fn cpp_sources<I, P>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.cpp_sources.extend(sources.into_iter().map(Into::into));
        self
    }

    /// Adds a C++ include directory.
    #[must_use]
    pub fn cpp_include(mut self, include_dir: impl Into<PathBuf>) -> Self {
        self.cpp_include_dirs.push(include_dir.into());
        self
    }

    /// Adds C++ include directories.
    #[must_use]
    pub fn cpp_includes<I, P>(mut self, include_dirs: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.cpp_include_dirs
            .extend(include_dirs.into_iter().map(Into::into));
        self
    }

    /// Adds a raw argument passed to Verilator.
    #[must_use]
    pub fn verilator_arg(mut self, argument: impl Into<OsString>) -> Self {
        self.verilator_arguments.push(argument.into());
        self
    }

    /// Adds raw arguments passed to Verilator.
    #[must_use]
    pub fn verilator_args<I, A>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<OsString>,
    {
        self.verilator_arguments
            .extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Overrides the Verilator executable used for this DUT.
    #[must_use]
    pub fn verilator_executable(mut self, executable: impl Into<OsString>) -> Self {
        self.verilator_executable = Some(executable.into());
        self
    }

    /// Generates the Verilated model and compiles the native bridge.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError`] if configuration, tool execution, or native compilation fails.
    pub fn build(self) -> BuildResult<()> {
        validate_identifier("DUT name", &self.name)?;

        let top_module = self.top_module.ok_or(BuildError::MissingTopModule)?;
        validate_identifier("top module", &top_module)?;

        if self.sources.is_empty() {
            return Err(BuildError::MissingSources);
        }

        validate_defines(&self.defines)?;

        let bridge = self.bridge.ok_or(BuildError::MissingBridge)?;

        let manifest_dir = required_environment_path("CARGO_MANIFEST_DIR")?;
        let out_dir = required_environment_path("OUT_DIR")?;

        let model_prefix = format!("V{}", self.name);
        let dut_output_dir = out_dir.join("vvm").join(&self.name);
        let verilated_dir = dut_output_dir.join("verilated");
        let metadata_dir = dut_output_dir.join("metadata");

        create_directory(&verilated_dir)?;
        create_directory(&metadata_dir)?;

        let sources = paths::resolve_files(&manifest_dir, &self.sources, "HDL source file")?;
        let hdl_include_dirs = paths::resolve_directories(
            &manifest_dir,
            &self.hdl_include_dirs,
            "HDL include directory",
        )?;
        let bridge = paths::resolve_file(&manifest_dir, &bridge, "CXX bridge source")?;
        let cpp_sources =
            paths::resolve_files(&manifest_dir, &self.cpp_sources, "C++ source file")?;
        let cpp_include_dirs = paths::resolve_directories(
            &manifest_dir,
            &self.cpp_include_dirs,
            "C++ include directory",
        )?;

        paths::ensure_unique_paths(&sources, "HDL source file")?;
        paths::ensure_unique_paths(&hdl_include_dirs, "HDL include directory")?;
        paths::ensure_unique_paths(std::slice::from_ref(&bridge), "CXX bridge source")?;
        paths::ensure_unique_paths(&cpp_sources, "C++ source file")?;
        paths::ensure_unique_paths(&cpp_include_dirs, "C++ include directory")?;

        cargo::emit_rerun_directives(
            &sources,
            &hdl_include_dirs,
            &bridge,
            &cpp_sources,
            &cpp_include_dirs,
        );

        let environment_executable = env::var_os("VERILATOR");
        let executable =
            verilator::select_executable(self.verilator_executable.clone(), environment_executable);

        let version = verilator::version(&executable)?;
        verilator::ensure_supported_version(version)?;

        let metadata_output = metadata_dir.join(format!("{}.tree.json", self.name));

        let metadata_meta_output = metadata_dir.join(format!("{}.tree.meta.json", self.name));

        let metadata_command = verilator::MetadataCommand {
            executable: &executable,
            top_module: &top_module,
            output: &metadata_output,
            meta_output: &metadata_meta_output,
            hdl_include_dirs: &hdl_include_dirs,
            defines: &self.defines,
            extra_arguments: &self.verilator_arguments,
            sources: &sources,
        };

        let metadata_files = verilator::generate_metadata(&metadata_command)?;

        let raw_metadata = crate::metadata::RawMetadata::from_paths(
            version,
            &metadata_files.tree,
            &metadata_files.meta,
        )?;

        let dut_metadata = crate::metadata::normalize(&self.name, &top_module, &raw_metadata)?;
        crate::metadata::validate_supported(&dut_metadata)?;

        let verilator_root = verilator::root(&executable)?;

        let model_command = verilator::ModelCommand {
            executable: &executable,
            top_module: &top_module,
            model_prefix: &model_prefix,
            output_dir: &verilated_dir,
            hdl_include_dirs: &hdl_include_dirs,
            defines: &self.defines,
            extra_arguments: &self.verilator_arguments,
            sources: &sources,
        };

        verilator::generate(&model_command)?;

        let generated_sources = verilator::generated_sources(&verilated_dir, &model_prefix)?;

        native::compile(
            &self.name,
            &bridge,
            &cpp_sources,
            &cpp_include_dirs,
            &verilated_dir,
            &verilator_root,
            &generated_sources,
        )
    }

    #[cfg(test)]
    pub(crate) fn sources_slice(&self) -> &[PathBuf] {
        &self.sources
    }

    #[cfg(test)]
    pub(crate) fn hdl_include_dirs_slice(&self) -> &[PathBuf] {
        &self.hdl_include_dirs
    }

    #[cfg(test)]
    pub(crate) fn cpp_sources_slice(&self) -> &[PathBuf] {
        &self.cpp_sources
    }

    #[cfg(test)]
    pub(crate) fn cpp_include_dirs_slice(&self) -> &[PathBuf] {
        &self.cpp_include_dirs
    }

    #[cfg(test)]
    pub(crate) fn verilator_arguments_slice(&self) -> &[OsString] {
        &self.verilator_arguments
    }
}

/// Reads a required Cargo build-script environment path.
fn required_environment_path(name: &'static str) -> BuildResult<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironmentVariable { name })
}

/// Creates an output directory.
fn create_directory(path: &Path) -> BuildResult<()> {
    fs::create_dir_all(path).map_err(|source| BuildError::Io {
        operation: "create output directory",
        path: path.to_path_buf(),
        source,
    })
}

/// Checks whether a name can be safely used as a C++ identifier.
///
/// # Errors
/// Returns [`BuildError::InvalidIdentifier`] if the identifier is empty or not ASCII C/C++-style.
pub fn validate_identifier(field: &'static str, value: &str) -> BuildResult<()> {
    let mut characters = value.chars();

    let Some(first) = characters.next() else {
        return Err(BuildError::InvalidIdentifier {
            field,
            value: value.to_owned(),
        });
    };

    let valid_first = first == '_' || first.is_ascii_alphabetic();
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

/// Validates configured HDL definitions.
pub fn validate_defines(defines: &[Define]) -> BuildResult<()> {
    let mut seen_names = std::collections::HashSet::new();

    for define in defines {
        validate_identifier("HDL definition", &define.name)?;

        if !seen_names.insert(define.name.clone()) {
            return Err(BuildError::DuplicateDefine {
                name: define.name.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
impl Define {
    pub(crate) fn new(name: &str, value: Option<&str>) -> Self {
        Self {
            name: name.to_owned(),
            value: value.map(str::to_owned),
        }
    }
}

#[cfg(test)]
impl DutBuilder {
    pub(crate) fn defines_slice(&self) -> &[Define] {
        &self.defines
    }
}

#[cfg(test)]
pub const fn minimum_supported_version() -> crate::verilator::VerilatorVersion {
    verilator::minimum_supported_version()
}
