/// C++ adapter source generation.
pub mod cpp_adapter;
/// Raw CXX bridge source generation.
pub mod cxx_bridge;
/// Deterministic generated C++ naming.
pub mod names;
/// Safe Rust DUT wrapper generation.
pub mod rust_wrapper;
/// Generated signal type selection.
pub mod types;
/// Deterministic generated-file writing.
pub mod writer;

use std::fs;
use std::path::{Path, PathBuf};

use crate::metadata::DutMetadata;
use crate::{BuildError, BuildResult};

/// Generated DUT integration artifacts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedArtifacts {
    /// Directory containing the generated public header.
    pub(crate) include_dir: PathBuf,

    /// Generated C++ implementation source.
    pub(crate) cpp_source: PathBuf,

    /// Generated raw CXX bridge.
    pub(crate) cxx_bridge: PathBuf,

    /// Safe Rust DUT wrapper generation.
    pub(crate) rust_wrapper: PathBuf,
}

/// Generates the C++ adapter for one normalized DUT.
///
/// # Errors
///
/// Returns an error when generated names are invalid or output files cannot
/// be created.
pub fn generate(
    metadata: &DutMetadata,
    model_prefix: &str,
    output_dir: &Path,
) -> BuildResult<GeneratedArtifacts> {
    let names = names::resolve(metadata, model_prefix)?;
    let adapter = cpp_adapter::render(metadata, &names);
    let bridge = cxx_bridge::render(metadata, &names);
    let wrapper = rust_wrapper::render(metadata, &names);

    fs::create_dir_all(output_dir).map_err(|source| BuildError::Io {
        operation: "create generated source directory",
        path: output_dir.to_path_buf(),
        source,
    })?;

    let header = output_dir.join(format!("{}.hpp", names.file_stem));
    let cpp_source = output_dir.join(format!("{}.cpp", names.file_stem));
    let cxx_bridge = output_dir.join("bridge.rs");
    let rust_wrapper = output_dir.join("dut.rs");

    writer::write_if_changed(&header, &adapter.header)?;
    writer::write_if_changed(&cpp_source, &adapter.source)?;
    writer::write_if_changed(&cxx_bridge, &bridge)?;
    writer::write_if_changed(&rust_wrapper, &wrapper)?;

    Ok(GeneratedArtifacts {
        include_dir: output_dir.to_path_buf(),
        cpp_source,
        cxx_bridge,
        rust_wrapper,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::generate;
    use crate::metadata::{RawMetadata, normalize};
    use crate::verilator::VerilatorVersion;

    #[test]
    fn generates_expected_counter_adapter() -> Result<(), Box<dyn std::error::Error>> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let metadata_fixture = manifest
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("counter");

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &metadata_fixture.join("counter.tree.json"),
            &metadata_fixture.join("counter.tree.meta.json"),
        )?;

        let metadata = normalize("counter", "counter", &raw)?;

        let output = tempdir()?;

        let generated = generate(&metadata, "Vcounter", output.path())?;

        let expected_directory = manifest
            .join("tests")
            .join("fixtures")
            .join("codegen")
            .join("counter");

        assert_eq!(
            std::fs::read_to_string(output.path().join("counter.hpp"))?,
            std::fs::read_to_string(expected_directory.join("counter.hpp"))?
        );

        assert_eq!(
            std::fs::read_to_string(generated.cpp_source)?,
            std::fs::read_to_string(expected_directory.join("counter.cpp"))?
        );

        assert_eq!(
            std::fs::read_to_string(generated.cxx_bridge)?,
            std::fs::read_to_string(expected_directory.join("bridge.rs"))?
        );

        assert_eq!(
            std::fs::read_to_string(generated.rust_wrapper)?,
            std::fs::read_to_string(expected_directory.join("dut.rs"))?
        );

        Ok(())
    }
}
