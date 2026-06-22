use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::path::Path;

use serde_json::{Map, Value};

use crate::metadata::{BitWidth, DutMetadata, Port, PortDirection, RawMetadata};
use crate::{BuildError, BuildResult};

/// Main Verilator AST metadata role.
const TREE_ROLE: &str = "Verilator AST metadata";

/// Maximum packed width supported by the current generated bridge.
const MAXIMUM_SUPPORTED_WIDTH: u32 = 64;

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

/// Normalizes raw Verilator metadata into a stable DUT model.
///
/// # Errors
///
/// Returns an error if the requested module cannot be found, the AST has an
/// unexpected shape, references cannot be resolved, or a datatype cannot be
/// normalized.
pub fn normalize(dut_name: &str, top_module: &str, raw: &RawMetadata) -> BuildResult<DutMetadata> {
    let root = raw.tree_root();
    let path = raw.tree_path();

    let index = AstIndex::build(root, path)?;
    let module = find_top_module(root, top_module, path)?;
    let statements = required_array(module, "stmtsp", path)?;

    let mut ports = Vec::new();
    let mut port_names = HashSet::new();

    for statement in statements {
        let statement =
            statement
                .as_object()
                .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                    role: TREE_ROLE,
                    path: path.to_path_buf(),
                    field: "stmtsp element",
                    expected: "an object",
                })?;

        if optional_string(statement, "type", path)? != Some("VAR") {
            continue;
        }

        if optional_string(statement, "varType", path)? != Some("PORT") {
            continue;
        }

        let port = normalize_port(statement, &index, path)?;

        if !port_names.insert(port.name.clone()) {
            return Err(BuildError::DuplicatePortName {
                top_module: top_module.to_owned(),
                port: port.name,
            });
        }

        ports.push(port);
    }

    Ok(DutMetadata {
        name: dut_name.to_owned(),
        top_module: top_module.to_owned(),
        ports,
    })
}

/// Verifies that normalized metadata is supported by the current bridge.
///
/// Normalization itself is descriptive: it can represent inout, signed, and
/// wide ports. This function applies the current implementation limits.
///
/// # Errors
///
/// Returns an error for inout ports, signed ports, or widths above 64 bits.
pub fn validate_supported(metadata: &DutMetadata) -> BuildResult<()> {
    for port in &metadata.ports {
        if port.direction == PortDirection::Inout {
            return Err(BuildError::UnsupportedInoutPort {
                port: port.name.clone(),
            });
        }

        if port.signed {
            return Err(BuildError::UnsupportedSignedPort {
                port: port.name.clone(),
            });
        }

        let width = port.width.get();

        if width > MAXIMUM_SUPPORTED_WIDTH {
            return Err(BuildError::UnsupportedPortWidth {
                port: port.name.clone(),
                width,
                maximum: MAXIMUM_SUPPORTED_WIDTH,
            });
        }
    }

    Ok(())
}

/// Finds exactly one matching direct module definition.
fn find_top_module<'a>(
    root: &'a Map<String, Value>,
    top_module: &str,
    path: &Path,
) -> BuildResult<&'a Map<String, Value>> {
    let modules = required_array(root, "modulesp", path)?;

    let mut matching_module = None;

    for module in modules {
        let module = module
            .as_object()
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "modulesp element",
                expected: "an object",
            })?;

        if optional_string(module, "type", path)? != Some("MODULE") {
            continue;
        }

        if !module_matches(module, top_module, path)? {
            continue;
        }

        if matching_module.replace(module).is_some() {
            return Err(BuildError::DuplicateTopModuleMetadata {
                top_module: top_module.to_owned(),
                path: path.to_path_buf(),
            });
        }
    }

    matching_module.ok_or_else(|| BuildError::MissingTopModuleMetadata {
        top_module: top_module.to_owned(),
        path: path.to_path_buf(),
    })
}

/// Checks whether a module node represents the requested HDL module.
fn module_matches(module: &Map<String, Value>, top_module: &str, path: &Path) -> BuildResult<bool> {
    for field in ["origName", "verilogName", "name"] {
        if optional_string(module, field, path)? == Some(top_module) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Normalizes one top-level port declaration.
fn normalize_port(
    node: &Map<String, Value>,
    index: &AstIndex<'_>,
    path: &Path,
) -> BuildResult<Port> {
    let name = port_name(node, path)?.to_owned();
    let direction = normalize_direction(&name, required_string(node, "direction", path)?)?;

    let dtype_reference = required_string(node, "dtypep", path)?;

    let dtype =
        index
            .resolve(dtype_reference)
            .ok_or_else(|| BuildError::UnresolvedPortDataType {
                port: name.clone(),
                reference: dtype_reference.to_owned(),
                path: path.to_path_buf(),
            })?;

    let (width, signed) = normalize_data_type(&name, dtype, path)?;

    Ok(Port {
        name,
        direction,
        width,
        signed,
    })
}

/// Returns the HDL-visible name of a port.
fn port_name<'a>(node: &'a Map<String, Value>, path: &Path) -> BuildResult<&'a str> {
    if let Some(name) = optional_string(node, "verilogName", path)? {
        return Ok(name);
    }

    required_string(node, "name", path)
}

/// Normalizes a Verilator port direction.
fn normalize_direction(port: &str, direction: &str) -> BuildResult<PortDirection> {
    match direction {
        "INPUT" => Ok(PortDirection::Input),
        "OUTPUT" => Ok(PortDirection::Output),
        "INOUT" => Ok(PortDirection::Inout),
        other => Err(BuildError::UnknownPortDirection {
            port: port.to_owned(),
            direction: other.to_owned(),
        }),
    }
}

/// Normalizes a referenced datatype node.
fn normalize_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    path: &Path,
) -> BuildResult<(BitWidth, bool)> {
    let node_type = required_string(dtype, "type", path)?;

    if node_type != "BASICDTYPE" {
        return Err(BuildError::UnsupportedPortDataType {
            port: port.to_owned(),
            kind: node_type.to_owned(),
        });
    }

    normalize_basic_data_type(port, dtype, path)
}

/// Normalizes an integral Verilator `BASICDTYPE`.
fn normalize_basic_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    path: &Path,
) -> BuildResult<(BitWidth, bool)> {
    let keyword = required_string(dtype, "keyword", path)?;

    // Keep the first implementation deliberately narrow. Other BASICDTYPE
    // keywords have language-defined width and signedness defaults that should
    // be added through dedicated fixtures.
    if keyword != "logic" && keyword != "bit" {
        return Err(BuildError::UnsupportedPortDataType {
            port: port.to_owned(),
            kind: format!("BASICDTYPE({keyword})"),
        });
    }

    let range = optional_string(dtype, "range", path)?;
    let width = parse_bit_width(port, range)?;
    let signed = basic_data_type_signed(dtype, path)?;

    Ok((width, signed))
}

/// Reads signedness from a Verilator basic datatype.
///
/// Verilator versions may represent enum-like properties either as strings or
/// booleans. Absence means the unsigned/default representation for the
/// currently supported `logic` and `bit` fixtures.
fn basic_data_type_signed(dtype: &Map<String, Value>, path: &Path) -> BuildResult<bool> {
    let Some(value) = dtype.get("signed") else {
        return Ok(false);
    };

    match *value {
        Value::Bool(value) => Ok(value),
        Value::String(ref value) => match value.as_str() {
            "SIGNED" | "signed" => Ok(true),
            "UNSIGNED" | "unsigned" | "NOSIGN" => Ok(false),
            _ => Err(BuildError::InvalidMetadataFieldType {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "signed",
                expected: "a signedness boolean or recognized signedness string",
            }),
        },
        _ => Err(BuildError::InvalidMetadataFieldType {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field: "signed",
            expected: "a boolean or string",
        }),
    }
}

/// Converts a Verilator packed range into a non-zero width.
fn parse_bit_width(port: &str, range: Option<&str>) -> BuildResult<BitWidth> {
    let Some(range) = range else {
        return Ok(BitWidth::new(NonZeroU32::new(1).ok_or_else(|| {
            BuildError::InvalidPortRange {
                port: port.to_owned(),
                range: String::new(),
            }
        })?));
    };

    let Some((left, right)) = range.split_once(':') else {
        return Err(BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        });
    };

    let left = parse_range_bound(port, range, left)?;
    let right = parse_range_bound(port, range, right)?;

    let distance = left.abs_diff(right);
    let width = distance
        .checked_add(1)
        .ok_or_else(|| BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        })?;

    let width = u32::try_from(width).map_err(|_conversion_error| BuildError::InvalidPortRange {
        port: port.to_owned(),
        range: range.to_owned(),
    })?;

    let width = NonZeroU32::new(width).ok_or_else(|| BuildError::InvalidPortRange {
        port: port.to_owned(),
        range: range.to_owned(),
    })?;

    Ok(BitWidth::new(width))
}

/// Parses one numeric bound of a packed range.
fn parse_range_bound(port: &str, range: &str, bound: &str) -> BuildResult<i64> {
    bound
        .trim()
        .parse::<i64>()
        .map_err(|_parse_error| BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        })
}

/// Returns a required JSON array field.
fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
    path: &Path,
) -> BuildResult<&'a [Value]> {
    let value = object
        .get(field)
        .ok_or_else(|| BuildError::MissingMetadataField {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field,
        })?;

    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| BuildError::InvalidMetadataFieldType {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field,
            expected: "an array",
        })
}

/// Returns a required JSON string field.
fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
    path: &Path,
) -> BuildResult<&'a str> {
    let value = object
        .get(field)
        .ok_or_else(|| BuildError::MissingMetadataField {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field,
        })?;

    value
        .as_str()
        .ok_or_else(|| BuildError::InvalidMetadataFieldType {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field,
            expected: "a string",
        })
}

/// Returns an optional JSON string field.
fn optional_string<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
    path: &Path,
) -> BuildResult<Option<&'a str>> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };

    value
        .as_str()
        .map(Some)
        .ok_or_else(|| BuildError::InvalidMetadataFieldType {
            role: TREE_ROLE,
            path: path.to_path_buf(),
            field,
            expected: "a string",
        })
}

/// Visit AST value by recursively traversing the AST tree.
fn visit_value<'a>(
    value: &'a Value,
    path: &Path,
    nodes: &mut HashMap<&'a str, &'a Map<String, Value>>,
) -> BuildResult<()> {
    match *value {
        Value::Object(ref object) => visit_object(object, path, nodes),
        Value::Array(ref values) => {
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::path::PathBuf;

    use super::{normalize, parse_bit_width, validate_supported};
    use crate::error::BuildError;
    use crate::metadata::model::{BitWidth, DutMetadata, Port, PortDirection};
    use crate::metadata::raw::RawMetadata;
    use crate::verilator::VerilatorVersion;

    fn width(value: u32) -> BitWidth {
        let value = NonZeroU32::new(value).expect("test bit width must be non-zero");

        BitWidth::new(value)
    }

    fn counter_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("counter")
    }

    #[test]
    fn normalizes_counter_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = counter_fixture();

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &fixture.join("counter.tree.json"),
            &fixture.join("counter.tree.meta.json"),
        )?;

        let actual = normalize("counter", "counter", &raw)?;

        let expected = DutMetadata {
            name: "counter".to_owned(),
            top_module: "counter".to_owned(),
            ports: vec![
                Port {
                    name: "clk".to_owned(),
                    direction: PortDirection::Input,
                    width: width(1),
                    signed: false,
                },
                Port {
                    name: "reset_n".to_owned(),
                    direction: PortDirection::Input,
                    width: width(1),
                    signed: false,
                },
                Port {
                    name: "enable".to_owned(),
                    direction: PortDirection::Input,
                    width: width(1),
                    signed: false,
                },
                Port {
                    name: "count".to_owned(),
                    direction: PortDirection::Output,
                    width: width(8),
                    signed: false,
                },
            ],
        };

        assert_eq!(actual, expected);
        validate_supported(&actual)?;

        Ok(())
    }

    #[test]
    fn parses_descending_packed_range() -> Result<(), BuildError> {
        assert_eq!(parse_bit_width("value", Some("7:0"))?.get(), 8);

        Ok(())
    }

    #[test]
    fn parses_ascending_packed_range() -> Result<(), BuildError> {
        assert_eq!(parse_bit_width("value", Some("0:7"))?.get(), 8);

        Ok(())
    }

    #[test]
    fn scalar_range_defaults_to_one_bit() -> Result<(), BuildError> {
        assert_eq!(parse_bit_width("value", None)?.get(), 1);

        Ok(())
    }

    #[test]
    fn rejects_malformed_packed_range() {
        assert!(matches!(
            parse_bit_width("value", Some("7")),
            Err(BuildError::InvalidPortRange { .. })
        ));

        assert!(matches!(
            parse_bit_width("value", Some("left:right")),
            Err(BuildError::InvalidPortRange { .. })
        ));
    }

    #[test]
    fn rejects_currently_unsupported_inout_port() {
        let metadata = DutMetadata {
            name: "dut".to_owned(),
            top_module: "dut".to_owned(),
            ports: vec![Port {
                name: "bus".to_owned(),
                direction: PortDirection::Inout,
                width: width(1),
                signed: false,
            }],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedInoutPort { port })
                if port == "bus"
        ));
    }

    #[test]
    fn rejects_currently_unsupported_signed_port() {
        let metadata = DutMetadata {
            name: "dut".to_owned(),
            top_module: "dut".to_owned(),
            ports: vec![Port {
                name: "value".to_owned(),
                direction: PortDirection::Input,
                width: width(8),
                signed: true,
            }],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedSignedPort { port })
                if port == "value"
        ));
    }

    #[test]
    fn rejects_port_wider_than_sixty_four_bits() {
        let metadata = DutMetadata {
            name: "dut".to_owned(),
            top_module: "dut".to_owned(),
            ports: vec![Port {
                name: "value".to_owned(),
                direction: PortDirection::Output,
                width: width(65),
                signed: false,
            }],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedPortWidth {
                port,
                width: 65,
                maximum: 64,
            }) if port == "value"
        ));
    }
}
