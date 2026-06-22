//! Raw Verilator JSON metadata ingestion.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::error::{BuildError, BuildResult};
use crate::verilator::VerilatorVersion;

/// Main Verilator AST metadata document role.
const TREE_ROLE: &str = "Verilator AST metadata";

/// Companion Verilator metadata document role.
const META_ROLE: &str = "Verilator AST file metadata";

/// Raw metadata emitted by one Verilator invocation.
///
/// This type deliberately preserves the complete JSON documents. Interpretation
/// of AST nodes belongs to the normalization layer.
#[derive(Debug)]
pub struct RawMetadata {
    /// Version of Verilator that emitted these documents.
    version: VerilatorVersion,

    /// Main AST document.
    tree: RawDocument,

    /// Companion file and pointer metadata.
    meta: RawDocument,
}

impl RawMetadata {
    /// Reads and minimally validates generated metadata files.
    ///
    /// # Errors
    ///
    /// Returns an error if either file cannot be read, contains invalid JSON,
    /// or does not have the expected outer document shape.
    pub(crate) fn from_paths(
        version: VerilatorVersion,
        tree_path: &Path,
        meta_path: &Path,
    ) -> BuildResult<Self> {
        let metadata = Self {
            version,
            tree: RawDocument::from_path(tree_path, TREE_ROLE)?,
            meta: RawDocument::from_path(meta_path, META_ROLE)?,
        };

        metadata.validate_envelope()?;

        Ok(metadata)
    }

    /// Returns the producing verilator version.
    pub(crate) const fn version(&self) -> VerilatorVersion {
        self.version
    }

    /// Return the main AST document path.
    pub(crate) fn tree_path(&self) -> &Path {
        &self.tree.path
    }

    /// Returns the main AST root.
    pub(crate) const fn tree_root(&self) -> &Map<String, Value> {
        &self.tree.root
    }

    /// Validates only enough structure to identify the two documents.
    fn validate_envelope(&self) -> BuildResult<()> {
        let root_type = self.tree.required_string("type")?;

        if root_type != "NETLIST" {
            return Err(BuildError::InvalidMetadataFieldType {
                role: self.tree.role,
                path: self.tree.path.clone(),
                field: "type",
                expected: "the string `NETLIST`",
            });
        }

        // Reading these fields also verifies that the companion document has
        // the broad shape documented by Verilator. Their contents remain
        // uninterpreted in this milestone.
        let _files = self.meta.optional_object("files")?;
        let _pointers = self.meta.optional_object("pointers")?;
        let _pointer_fields = self.meta.optional_array("ptrFieldNames")?;

        // Keep the producing version associated with the documents. It will
        // become relevant if normalization needs version-specific handling.
        let _ = self.version;

        Ok(())
    }

    #[cfg(test)]
    const fn meta_root(&self) -> &Map<String, Value> {
        &self.meta.root
    }
}

/// One raw JSON document with source context.
#[derive(Debug)]
struct RawDocument {
    /// Human-readable document role.
    role: &'static str,

    /// Source path.
    path: PathBuf,

    /// Complete root JSON object.
    root: Map<String, Value>,
}

impl RawDocument {
    /// Reads a JSON object from a file.
    fn from_path(path: &Path, role: &'static str) -> BuildResult<Self> {
        let file = File::open(path).map_err(|source| BuildError::MetadataRead {
            role,
            path: path.to_path_buf(),
            source,
        })?;

        let reader = BufReader::new(file);

        let value: Value =
            serde_json::from_reader(reader).map_err(|source| BuildError::InvalidMetadataJson {
                role,
                path: path.to_path_buf(),
                source,
            })?;

        let Value::Object(root) = value else {
            return Err(BuildError::MetadataRootNotObject {
                role,
                path: path.to_path_buf(),
            });
        };

        Ok(Self {
            role,
            path: path.to_path_buf(),
            root,
        })
    }

    /// Returns a required string field.
    fn required_string(&self, field: &'static str) -> BuildResult<&str> {
        let value = self
            .root
            .get(field)
            .ok_or_else(|| BuildError::MissingMetadataField {
                role: self.role,
                path: self.path.clone(),
                field,
            })?;

        value
            .as_str()
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: self.role,
                path: self.path.clone(),
                field,
                expected: "a string",
            })
    }

    /// Returns an optional object field.
    fn optional_object(&self, field: &'static str) -> BuildResult<Option<&Map<String, Value>>> {
        let Some(value) = self.root.get(field) else {
            return Ok(None);
        };

        value
            .as_object()
            .map(Some)
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: self.role,
                path: self.path.clone(),
                field,
                expected: "an object",
            })
    }

    /// Returns an optional array field.
    fn optional_array(&self, field: &'static str) -> BuildResult<Option<&[Value]>> {
        let Some(value) = self.root.get(field) else {
            return Ok(None);
        };

        value
            .as_array()
            .map(Vec::as_slice)
            .map(Some)
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: self.role,
                path: self.path.clone(),
                field,
                expected: "an array",
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    use super::RawMetadata;
    use crate::error::BuildError;
    use crate::verilator::VerilatorVersion;

    fn write(path: &Path, contents: &str) -> Result<(), std::io::Error> {
        fs::write(path, contents)
    }

    #[test]
    fn parses_minimal_raw_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let tree = directory.path().join("tree.json");
        let meta = directory.path().join("meta.json");

        write(
            &tree,
            r#"{
                "type": "NETLIST",
                "futureField": {
                    "unknown": true
                }
            }"#,
        )?;

        write(
            &meta,
            r#"{
                "files": {},
                "pointers": {},
                "ptrFieldNames": []
            }"#,
        )?;

        let raw = RawMetadata::from_paths(VerilatorVersion::new(5, 48), &tree, &meta)?;

        assert!(raw.tree_root().contains_key("futureField"));
        assert!(raw.meta_root().contains_key("files"));

        Ok(())
    }

    #[test]
    fn rejects_malformed_tree_json() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let tree = directory.path().join("tree.json");
        let meta = directory.path().join("meta.json");

        write(&tree, r#"{"type":"NETLIST""#)?;
        write(&meta, "{}")?;

        assert!(matches!(
            RawMetadata::from_paths(VerilatorVersion::new(5, 48), &tree, &meta,),
            Err(BuildError::InvalidMetadataJson { .. })
        ));

        Ok(())
    }

    #[test]
    fn rejects_non_object_root() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let tree = directory.path().join("tree.json");
        let meta = directory.path().join("meta.json");

        write(&tree, "[]")?;
        write(&meta, "{}")?;

        assert!(matches!(
            RawMetadata::from_paths(VerilatorVersion::new(5, 48), &tree, &meta,),
            Err(BuildError::MetadataRootNotObject { .. })
        ));

        Ok(())
    }

    #[test]
    fn rejects_missing_tree_type() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let tree = directory.path().join("tree.json");
        let meta = directory.path().join("meta.json");

        write(&tree, "{}")?;
        write(&meta, "{}")?;

        assert!(matches!(
            RawMetadata::from_paths(VerilatorVersion::new(5, 48), &tree, &meta,),
            Err(BuildError::MissingMetadataField { field: "type", .. })
        ));

        Ok(())
    }

    #[test]
    fn rejects_wrong_tree_root_type() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let tree = directory.path().join("tree.json");
        let meta = directory.path().join("meta.json");

        write(&tree, r#"{"type":"MODULE"}"#)?;
        write(&meta, "{}")?;

        assert!(matches!(
            RawMetadata::from_paths(VerilatorVersion::new(5, 48), &tree, &meta,),
            Err(BuildError::InvalidMetadataFieldType { field: "type", .. })
        ));

        Ok(())
    }

    #[test]
    fn parses_versioned_counter_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("counter");

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &fixture.join("counter.tree.json"),
            &fixture.join("counter.tree.meta.json"),
        )?;

        assert_eq!(
            raw.tree_root()
                .get("type")
                .and_then(serde_json::Value::as_str),
            Some("NETLIST")
        );

        assert!(raw
            .meta_root()
            .get("files")
            .is_some_and(serde_json::Value::is_object));

        Ok(())
    }
}
