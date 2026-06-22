use std::{collections::HashMap, path::Path};

use serde_json::{Map, Value};

use crate::{BuildError, BuildResult};

/// Lookup table for Verilator AST short addresses.
struct AstIndex<'a> {
    /// AST tree node address map.
    nodes: HashMap<&'a str, &'a Map<String, Value>>,
}

impl<'a> AstIndex<'a> {
    /// Build AST address lookup table.
    fn build(root: &'a Map<String, Value>, path: &Path) -> BuildResult<Self> {
        let mut nodes = HashMap::new();

        visit_object(root, path, &mut nodes)?;

        Ok(Self { nodes })
    }

    /// Resolve AST node by address reference.
    fn resolve(&self, reference: &str) -> Option<&'a Map<String, Value>> {
        self.nodes.get(reference).copied()
    }
}

/// Visit AST value by recursively traversing the AST tree.
fn visit_value<'a>(
    value: &'a Value,
    path: &Path,
    nodes: &mut HashMap<&'a str, &'a Map<String, Value>>,
) -> BuildResult<()> {
    match value {
        Value::Object(object) => visit_object(object, path, nodes),
        Value::Array(values) => {
            for json_value in values {
                visit_value(json_value, path, nodes)?;
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

/// Visit AST tree object by recursively traversing tree.
fn visit_object<'a>(
    object: &'a Map<String, Value>,
    path: &Path,
    nodes: &mut HashMap<&'a str, &'a Map<String, Value>>,
) -> BuildResult<()> {
    if let Some(address) = object.get("addr").and_then(Value::as_str) {
        if nodes.insert(address, object).is_some() {
            return Err(BuildError::DuplicateMetadataAddress {
                address: address.to_owned(),
                path: path.to_path_buf(),
            });
        }
    }

    for value in object.values() {
        visit_value(value, path, nodes)?;
    }

    Ok(())
}
