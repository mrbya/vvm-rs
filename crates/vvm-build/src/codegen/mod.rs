/// C++ adapter source generation.
pub mod cpp_adapter;
/// Deterministic generated C++ naming.
pub mod names;
/// Generated signal type selection.
pub mod types;
/// Deterministic generated-file writing.
pub mod writer;

use std::fs;
use std::path::{Path, PathBuf};

use crate::metadata::DutMetadata;
use crate::{BuildError, BuildResult};

/// Generated C++ adapter artifacts needed by native compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCppAdapter {
    /// Directory containing the generated public header.
    pub(crate) include_dir: PathBuf,

    /// Generated C++ implementation source.
    pub(crate) source: PathBuf,
}

/// Generates the C++ adapter for one normalized DUT.
///
/// # Errors
///
/// Returns an error when generated names are invalid or output files cannot
/// be created.
pub fn generate_cpp_adapter(
    metadata: &DutMetadata,
    model_prefix: &str,
    output_dir: &Path,
) -> BuildResult<GeneratedCppAdapter> {
    let names = names::resolve(metadata, model_prefix)?;
    let text = cpp_adapter::render(metadata, &names);

    fs::create_dir_all(output_dir).map_err(|source| BuildError::Io {
        operation: "create generated source directory",
        path: output_dir.to_path_buf(),
        source,
    })?;

    let header = output_dir.join(format!("{}.hpp", names.file_stem));
    let source = output_dir.join(format!("{}.cpp", names.file_stem));

    writer::write_if_changed(&header, &text.header)?;
    writer::write_if_changed(&source, &text.source)?;

    Ok(GeneratedCppAdapter {
        include_dir: output_dir.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::generate_cpp_adapter;
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

        let generated = generate_cpp_adapter(&metadata, "Vcounter", output.path())?;

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
            std::fs::read_to_string(generated.source)?,
            std::fs::read_to_string(expected_directory.join("counter.cpp"))?
        );

        Ok(())
    }
}
