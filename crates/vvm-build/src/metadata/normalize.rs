use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::path::Path;

use serde_json::{Map, Value};

use crate::metadata::{
    ArrayDimension, BitWidth, DutMetadata, PackedArrayShape, PackedEnumShape, PackedEnumVariant,
    PackedScalarShape, PackedStructField, PackedStructShape, Port, PortDirection, PortShape,
    RawMetadata, UnpackedArrayShape,
};
use crate::{BuildError, BuildResult};

/// Main Verilator AST metadata role.
const TREE_ROLE: &str = "Verilator AST metadata";

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
/// Returns an error for inout ports and aggregate port shapes.
pub fn validate_supported(metadata: &DutMetadata) -> BuildResult<()> {
    for port in &metadata.ports {
        if port.direction == PortDirection::Inout {
            return Err(BuildError::UnsupportedInoutPort {
                port: port.name.clone(),
            });
        }

        if port.shape.is_plain_packed_scalar() {
            continue;
        }

        if matches!(&port.shape, &PortShape::PackedArray(_)) {
            return Err(BuildError::UnsupportedPackedArrayPort {
                port: port.name.clone(),
            });
        }

        if matches!(&port.shape, &PortShape::PackedStruct(_)) {
            return Err(BuildError::UnsupportedPackedStructPort {
                port: port.name.clone(),
            });
        }

        if matches!(&port.shape, &PortShape::PackedEnum(_)) {
            return Err(BuildError::UnsupportedPackedEnumPort {
                port: port.name.clone(),
            });
        }

        if matches!(&port.shape, &PortShape::UnpackedArray(_)) {
            return Err(BuildError::UnsupportedUnpackedArrayPort {
                port: port.name.clone(),
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

    let shape = normalize_data_type(&name, dtype, index, path)?;
    let width = compatibility_width(&name, &shape)?;
    let signed = shape.signed();

    Ok(Port {
        name,
        direction,
        width,
        signed,
        shape,
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
    index: &AstIndex<'_>,
    path: &Path,
) -> BuildResult<PortShape> {
    let node_type = required_string(dtype, "type", path)?;

    match node_type {
        "BASICDTYPE" => normalize_basic_data_type(port, dtype, path),
        "PACKARRAYDTYPE" => normalize_array_data_type(port, dtype, index, path, true),
        "UNPACKARRAYDTYPE" => normalize_array_data_type(port, dtype, index, path, false),
        "STRUCTDTYPE" => normalize_struct_data_type(port, dtype, index, path),
        "ENUMDTYPE" => normalize_enum_data_type(port, dtype, index, path),
        "REFDTYPE" => {
            let reference = required_string(dtype, "refDTypep", path)?;
            let resolved = resolve_dtype_reference(port, reference, index, path)?;

            normalize_data_type(port, resolved, index, path)
        }
        other => Err(BuildError::UnsupportedPortDataType {
            port: port.to_owned(),
            kind: other.to_owned(),
        }),
    }
}

/// Normalizes an integral Verilator `BASICDTYPE`.
fn normalize_basic_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    path: &Path,
) -> BuildResult<PortShape> {
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

    Ok(PortShape::PackedScalar(PackedScalarShape { width, signed }))
}

/// Resolves one referenced dtype node.
fn resolve_dtype_reference<'a>(
    port: &str,
    reference: &str,
    index: &'a AstIndex<'_>,
    path: &Path,
) -> BuildResult<&'a Map<String, Value>> {
    index
        .resolve(reference)
        .ok_or_else(|| BuildError::UnresolvedPortDataType {
            port: port.to_owned(),
            reference: reference.to_owned(),
            path: path.to_path_buf(),
        })
}

/// Normalizes one Verilator array dtype node.
fn normalize_array_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    index: &AstIndex<'_>,
    path: &Path,
    packed: bool,
) -> BuildResult<PortShape> {
    let reference = required_string(dtype, "refDTypep", path)?;
    let element_dtype = resolve_dtype_reference(port, reference, index, path)?;
    let element_shape = normalize_data_type(port, element_dtype, index, path)?;

    let decl_range = required_string(dtype, "declRange", path)?;
    let dimension = parse_decl_dimension(port, decl_range)?;

    if packed {
        let element_width = compatibility_width(port, &element_shape)?;
        let width = checked_multiply_width(port, element_width, dimension.length)?;
        let signed = element_shape.signed();

        return Ok(match element_shape {
            PortShape::PackedArray(mut shape) => {
                let mut dimensions = vec![dimension];
                dimensions.append(&mut shape.dimensions);

                PortShape::PackedArray(PackedArrayShape {
                    element: shape.element,
                    dimensions,
                    width: checked_multiply_width(port, shape.width, dimension.length)?,
                    signed,
                })
            }
            shape => PortShape::PackedArray(PackedArrayShape {
                element: Box::new(shape),
                dimensions: vec![dimension],
                width,
                signed,
            }),
        });
    }

    Ok(match element_shape {
        PortShape::UnpackedArray(mut shape) => {
            let mut dimensions = vec![dimension];
            dimensions.append(&mut shape.dimensions);

            PortShape::UnpackedArray(UnpackedArrayShape {
                element: shape.element,
                dimensions,
            })
        }
        shape => PortShape::UnpackedArray(UnpackedArrayShape {
            element: Box::new(shape),
            dimensions: vec![dimension],
        }),
    })
}

/// Normalizes one Verilator packed struct dtype node.
fn normalize_struct_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    index: &AstIndex<'_>,
    path: &Path,
) -> BuildResult<PortShape> {
    let members = required_array(dtype, "membersp", path)?;
    let signed = metadata_signed(dtype, path)?;

    let mut fields = Vec::with_capacity(members.len());
    let mut offset = 0_u32;

    for member in members.iter().rev() {
        let member = member
            .as_object()
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "membersp element",
                expected: "an object",
            })?;

        if optional_string(member, "type", path)? != Some("MEMBERDTYPE") {
            return Err(BuildError::UnsupportedPortDataType {
                port: port.to_owned(),
                kind: String::from("STRUCTDTYPE(member)"),
            });
        }

        let name = required_string(member, "name", path)?.to_owned();
        let reference = required_string(member, "refDTypep", path)?;
        let member_dtype = resolve_dtype_reference(port, reference, index, path)?;
        let shape = normalize_data_type(port, member_dtype, index, path)?;
        let width = compatibility_width(port, &shape)?;
        let field_signed = shape.signed();

        fields.push(PackedStructField {
            name,
            shape,
            lsb_offset: offset,
            width,
            signed: field_signed,
        });

        offset = offset
            .checked_add(width.get())
            .ok_or_else(|| BuildError::InvalidPortRange {
                port: port.to_owned(),
                range: String::from("packed struct width overflow"),
            })?;
    }

    let total_width = NonZeroU32::new(offset).ok_or_else(|| BuildError::InvalidPortRange {
        port: port.to_owned(),
        range: String::from("zero-width packed struct"),
    })?;

    Ok(PortShape::PackedStruct(PackedStructShape {
        width: BitWidth::new(total_width),
        fields,
        signed,
    }))
}

/// Normalizes one Verilator packed enum dtype node.
fn normalize_enum_data_type(
    port: &str,
    dtype: &Map<String, Value>,
    index: &AstIndex<'_>,
    path: &Path,
) -> BuildResult<PortShape> {
    let reference = required_string(dtype, "refDTypep", path)?;
    let storage_dtype = resolve_dtype_reference(port, reference, index, path)?;
    let storage_shape = normalize_data_type(port, storage_dtype, index, path)?;
    let width = compatibility_width(port, &storage_shape)?;
    let signed = storage_shape.signed();
    let items = required_array(dtype, "itemsp", path)?;

    let mut variants = Vec::with_capacity(items.len());

    for item in items {
        let item = item
            .as_object()
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "itemsp element",
                expected: "an object",
            })?;

        if optional_string(item, "type", path)? != Some("ENUMITEM") {
            return Err(BuildError::UnsupportedPortDataType {
                port: port.to_owned(),
                kind: String::from("ENUMDTYPE(item)"),
            });
        }

        let name = required_string(item, "name", path)?.to_owned();
        let values = required_array(item, "valuep", path)?;
        let value = values
            .first()
            .ok_or_else(|| BuildError::MissingMetadataField {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "valuep element",
            })?
            .as_object()
            .ok_or_else(|| BuildError::InvalidMetadataFieldType {
                role: TREE_ROLE,
                path: path.to_path_buf(),
                field: "valuep element",
                expected: "an object",
            })?;

        let raw_value = parse_const_value(required_string(value, "name", path)?)?;

        variants.push(PackedEnumVariant {
            name,
            value: raw_value,
        });
    }

    Ok(PortShape::PackedEnum(PackedEnumShape {
        width,
        signed,
        variants,
    }))
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

/// Reads optional dtype signedness when aggregate nodes expose it.
fn metadata_signed(dtype: &Map<String, Value>, path: &Path) -> BuildResult<bool> {
    basic_data_type_signed(dtype, path)
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

/// Converts a Verilator array range into one normalized dimension.
fn parse_dimension(port: &str, range: &str) -> BuildResult<ArrayDimension> {
    let Some((left, right)) = range.split_once(':') else {
        return Err(BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        });
    };

    let left = parse_range_bound(port, range, left)?;
    let right = parse_range_bound(port, range, right)?;

    let distance = left.abs_diff(right);
    let length = distance
        .checked_add(1)
        .ok_or_else(|| BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        })?;

    let length =
        u32::try_from(length).map_err(|_conversion_error| BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        })?;

    let length = NonZeroU32::new(length).ok_or_else(|| BuildError::InvalidPortRange {
        port: port.to_owned(),
        range: range.to_owned(),
    })?;

    Ok(ArrayDimension {
        left,
        right,
        length,
    })
}

/// Parses a Verilator bracketed declaration range like `[3:0]`.
fn parse_decl_dimension(port: &str, range: &str) -> BuildResult<ArrayDimension> {
    let range = range
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| BuildError::InvalidPortRange {
            port: port.to_owned(),
            range: range.to_owned(),
        })?;

    parse_dimension(port, range)
}

/// Multiplies a packed width by an element count with overflow checking.
fn checked_multiply_width(port: &str, width: BitWidth, count: NonZeroU32) -> BuildResult<BitWidth> {
    let bits =
        width
            .get()
            .checked_mul(count.get())
            .ok_or_else(|| BuildError::InvalidPortRange {
                port: port.to_owned(),
                range: format!("{} * {}", width.get(), count),
            })?;

    let bits = NonZeroU32::new(bits).ok_or_else(|| BuildError::InvalidPortRange {
        port: port.to_owned(),
        range: String::from("zero-width aggregate"),
    })?;

    Ok(BitWidth::new(bits))
}

/// Returns flattened compatibility width used by existing scalar/wide codegen.
fn compatibility_width(port: &str, shape: &PortShape) -> BuildResult<BitWidth> {
    if let Some(width) = shape.packed_width() {
        return Ok(width);
    }

    debug_assert!(shape.is_aggregate());

    match *shape {
        PortShape::UnpackedArray(ref shape) => compatibility_width_for_unpacked_array(port, shape),
        PortShape::PackedScalar(_)
        | PortShape::PackedArray(_)
        | PortShape::PackedStruct(_)
        | PortShape::PackedEnum(_) => Err(BuildError::UnsupportedPortDataType {
            port: port.to_owned(),
            kind: String::from("shape width resolution failed"),
        }),
    }
}

/// Returns a flattened compatibility width for unsupported unpacked arrays.
fn compatibility_width_for_unpacked_array(
    port: &str,
    shape: &UnpackedArrayShape,
) -> BuildResult<BitWidth> {
    let mut width = compatibility_width(port, &shape.element)?;

    for dimension in &shape.dimensions {
        width = checked_multiply_width(port, width, dimension.length)?;
    }

    Ok(width)
}

/// Parses a Verilator constant literal like `2'h3` into a raw bit pattern.
fn parse_const_value(literal: &str) -> BuildResult<u64> {
    let Some((_, digits)) = literal.split_once('h') else {
        let Some((_, digits)) = literal.split_once('d') else {
            let Some((_, digits)) = literal.split_once('b') else {
                return digits_only_const_value(literal);
            };

            return u64::from_str_radix(digits, 2).map_err(|_parse_error| {
                BuildError::InvalidPortRange {
                    port: String::from("enum"),
                    range: literal.to_owned(),
                }
            });
        };

        return digits
            .parse::<u64>()
            .map_err(|_parse_error| BuildError::InvalidPortRange {
                port: String::from("enum"),
                range: literal.to_owned(),
            });
    };

    u64::from_str_radix(digits, 16).map_err(|_parse_error| BuildError::InvalidPortRange {
        port: String::from("enum"),
        range: literal.to_owned(),
    })
}

/// Parses a plain integer literal without an explicit Verilog base.
fn digits_only_const_value(literal: &str) -> BuildResult<u64> {
    literal
        .parse::<u64>()
        .map_err(|_parse_error| BuildError::InvalidPortRange {
            port: String::from("enum"),
            range: literal.to_owned(),
        })
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
    use crate::metadata::model::{
        ArrayDimension, BitWidth, DutMetadata, PackedArrayShape, PackedEnumShape,
        PackedEnumVariant, PackedScalarShape, PackedStructField, PackedStructShape, Port,
        PortDirection, PortShape, UnpackedArrayShape,
    };
    use crate::metadata::raw::RawMetadata;
    use crate::verilator::VerilatorVersion;

    fn width(value: u32) -> BitWidth {
        let value = NonZeroU32::new(value).expect("test bit width must be non-zero");

        BitWidth::new(value)
    }

    fn dimension(left: i64, right: i64, length: u32) -> ArrayDimension {
        ArrayDimension {
            left,
            right,
            length: NonZeroU32::new(length).expect("test dimension length must be non-zero"),
        }
    }

    fn scalar_port(name: &str, direction: PortDirection, bit_width: u32, signed: bool) -> Port {
        let width = width(bit_width);

        Port {
            name: name.to_owned(),
            direction,
            width,
            signed,
            shape: PortShape::PackedScalar(PackedScalarShape { width, signed }),
        }
    }

    fn aggregate_port(
        name: &str,
        direction: PortDirection,
        width: BitWidth,
        signed: bool,
        shape: PortShape,
    ) -> Port {
        Port {
            name: name.to_owned(),
            direction,
            width,
            signed,
            shape,
        }
    }

    fn aggregate_ports_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("aggregate_ports")
    }

    fn aggregate_ports_metadata() -> Result<DutMetadata, Box<dyn std::error::Error>> {
        let fixture = aggregate_ports_fixture();

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &fixture.join("aggregate_ports.tree.json"),
            &fixture.join("aggregate_ports.tree.meta.json"),
        )?;

        Ok(normalize("aggregate_ports", "aggregate_ports", &raw)?)
    }

    fn find_port<'a>(
        metadata: &'a DutMetadata,
        name: &str,
    ) -> Result<&'a Port, Box<dyn std::error::Error>> {
        metadata
            .ports
            .iter()
            .find(|port| port.name == name)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("missing port `{name}` in normalized metadata"),
                )
                .into()
            })
    }

    fn counter_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("counter")
    }

    fn signed_ports_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("signed_ports")
    }

    fn wide_ports_fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("verilator")
            .join("5.048")
            .join("wide_ports")
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
                scalar_port("clk", PortDirection::Input, 1, false),
                scalar_port("reset_n", PortDirection::Input, 1, false),
                scalar_port("enable", PortDirection::Input, 1, false),
                scalar_port("count", PortDirection::Output, 8, false),
            ],
        };

        assert_eq!(actual, expected);
        validate_supported(&actual)?;

        Ok(())
    }

    #[test]
    fn normalizes_signed_ports_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = signed_ports_fixture();

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &fixture.join("signed_ports.tree.json"),
            &fixture.join("signed_ports.tree.meta.json"),
        )?;

        let actual = normalize("signed_ports", "signed_ports", &raw)?;
        let expected_ports = [
            ("input_i1", PortDirection::Input, 1),
            ("input_i5", PortDirection::Input, 5),
            ("input_i9", PortDirection::Input, 9),
            ("input_i17", PortDirection::Input, 17),
            ("input_i33", PortDirection::Input, 33),
            ("input_i64", PortDirection::Input, 64),
            ("output_i1", PortDirection::Output, 1),
            ("output_i5", PortDirection::Output, 5),
            ("output_i9", PortDirection::Output, 9),
            ("output_i17", PortDirection::Output, 17),
            ("output_i33", PortDirection::Output, 33),
            ("output_i64", PortDirection::Output, 64),
        ]
        .into_iter()
        .map(|(name, direction, bit_width)| scalar_port(name, direction, bit_width, true))
        .collect();

        let expected = DutMetadata {
            name: "signed_ports".to_owned(),
            top_module: "signed_ports".to_owned(),
            ports: expected_ports,
        };

        assert_eq!(actual, expected);

        validate_supported(&actual)?;

        Ok(())
    }

    #[test]
    fn normalizes_wide_ports_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = wide_ports_fixture();

        let raw = RawMetadata::from_paths(
            VerilatorVersion::new(5, 48),
            &fixture.join("wide_ports.tree.json"),
            &fixture.join("wide_ports.tree.meta.json"),
        )?;

        let actual = normalize("wide_ports", "wide_ports", &raw)?;

        let expected_ports = [
            ("input_u65", PortDirection::Input, 65, false),
            ("input_u96", PortDirection::Input, 96, false),
            ("input_u129", PortDirection::Input, 129, false),
            ("input_u256", PortDirection::Input, 256, false),
            ("input_i65", PortDirection::Input, 65, true),
            ("input_i129", PortDirection::Input, 129, true),
            ("output_u65", PortDirection::Output, 65, false),
            ("output_u96", PortDirection::Output, 96, false),
            ("output_u129", PortDirection::Output, 129, false),
            ("output_u256", PortDirection::Output, 256, false),
            ("output_i65", PortDirection::Output, 65, true),
            ("output_i129", PortDirection::Output, 129, true),
        ]
        .into_iter()
        .map(|(name, direction, bit_width, signed)| scalar_port(name, direction, bit_width, signed))
        .collect();

        let expected = DutMetadata {
            name: "wide_ports".to_owned(),
            top_module: "wide_ports".to_owned(),
            ports: expected_ports,
        };

        assert_eq!(actual, expected);

        validate_supported(&actual)?;

        Ok(())
    }

    #[test]
    fn verilator_wide_accessors_use_vlwide_storage() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = wide_ports_fixture();
        let header = std::fs::read_to_string(fixture.join("Vwide_ports.h"))?;

        assert!(header.contains("VL_INW(&__Vm_sig_input_u65,64,0,3);"));
        assert!(header.contains("VL_INW(&__Vm_sig_input_u96,95,0,3);"));
        assert!(header.contains("VL_INW(&__Vm_sig_input_u129,128,0,5);"));
        assert!(header.contains("VL_INW(&__Vm_sig_input_u256,255,0,8);"));
        assert!(header.contains("VL_INW(&__Vm_sig_input_i65,64,0,3);"));
        assert!(header.contains("VL_INW(&__Vm_sig_input_i129,128,0,5);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_u65,64,0,3);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_u96,95,0,3);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_u129,128,0,5);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_u256,255,0,8);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_i65,64,0,3);"));
        assert!(header.contains("VL_OUTW(&__Vm_sig_output_i129,128,0,5);"));

        assert!(
            header
                .contains("decltype(__Vm_sig_input_u65) input_u65() {return __Vm_sig_input_u65;}")
        );
        assert!(
            header
                .contains("void input_u65(decltype(__Vm_sig_input_u65) v) {__Vm_sig_input_u65=v;}")
        );
        assert!(header.contains(
            "decltype(__Vm_sig_output_u256) output_u256() {return __Vm_sig_output_u256;}"
        ));
        assert!(header.contains(
            "void output_u256(decltype(__Vm_sig_output_u256) v) {__Vm_sig_output_u256=v;}"
        ));
        assert!(header.contains(
            "decltype(__Vm_sig_output_i129) output_i129() {return __Vm_sig_output_i129;}"
        ));
        assert!(header.contains(
            "void output_i129(decltype(__Vm_sig_output_i129) v) {__Vm_sig_output_i129=v;}"
        ));

        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn normalizes_aggregate_ports_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let actual = aggregate_ports_metadata()?;

        let clk = find_port(&actual, "clk")?;
        assert_eq!(clk.direction, PortDirection::Input);
        assert_eq!(clk.width, width(1));
        assert!(!clk.signed);
        assert_eq!(
            clk.shape,
            PortShape::PackedScalar(PackedScalarShape {
                width: width(1),
                signed: false,
            })
        );

        let packed_bytes = find_port(&actual, "packed_bytes")?;
        assert_eq!(packed_bytes.direction, PortDirection::Input);
        assert_eq!(packed_bytes.width, width(32));
        assert!(!packed_bytes.signed);
        assert_eq!(
            packed_bytes.shape,
            PortShape::PackedArray(PackedArrayShape {
                element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                    width: width(8),
                    signed: false,
                })),
                dimensions: vec![dimension(3, 0, 4)],
                width: width(32),
                signed: false,
            })
        );

        let packed_bytes_out = find_port(&actual, "packed_bytes_out")?;
        assert_eq!(packed_bytes_out.direction, PortDirection::Output);
        assert_eq!(packed_bytes_out.width, width(32));
        assert!(!packed_bytes_out.signed);

        let packet = find_port(&actual, "packet")?;
        assert_eq!(packet.direction, PortDirection::Input);
        assert_eq!(packet.width, width(16));
        assert!(!packet.signed);
        assert_eq!(
            packet.shape,
            PortShape::PackedStruct(PackedStructShape {
                width: width(16),
                fields: vec![
                    PackedStructField {
                        name: String::from("payload"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(8),
                            signed: false,
                        }),
                        lsb_offset: 0,
                        width: width(8),
                        signed: false,
                    },
                    PackedStructField {
                        name: String::from("flags"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(3),
                            signed: false,
                        }),
                        lsb_offset: 8,
                        width: width(3),
                        signed: false,
                    },
                    PackedStructField {
                        name: String::from("valid"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(1),
                            signed: false,
                        }),
                        lsb_offset: 11,
                        width: width(1),
                        signed: false,
                    },
                    PackedStructField {
                        name: String::from("opcode"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(4),
                            signed: false,
                        }),
                        lsb_offset: 12,
                        width: width(4),
                        signed: false,
                    },
                ],
                signed: false,
            })
        );

        let packet_out = find_port(&actual, "packet_out")?;
        assert_eq!(packet_out.direction, PortDirection::Output);
        assert_eq!(packet_out.width, width(16));
        assert!(!packet_out.signed);

        let state = find_port(&actual, "state")?;
        assert_eq!(state.direction, PortDirection::Input);
        assert_eq!(state.width, width(2));
        assert!(!state.signed);
        assert_eq!(
            state.shape,
            PortShape::PackedEnum(PackedEnumShape {
                width: width(2),
                signed: false,
                variants: vec![
                    PackedEnumVariant {
                        name: String::from("STATE_IDLE"),
                        value: 0,
                    },
                    PackedEnumVariant {
                        name: String::from("STATE_BUSY"),
                        value: 1,
                    },
                    PackedEnumVariant {
                        name: String::from("STATE_DONE"),
                        value: 2,
                    },
                    PackedEnumVariant {
                        name: String::from("STATE_ERR"),
                        value: 3,
                    },
                ],
            })
        );

        let state_out = find_port(&actual, "state_out")?;
        assert_eq!(state_out.direction, PortDirection::Output);
        assert_eq!(state_out.width, width(2));
        assert!(!state_out.signed);

        let unpacked_bytes = find_port(&actual, "unpacked_bytes")?;
        assert_eq!(unpacked_bytes.direction, PortDirection::Input);
        assert_eq!(unpacked_bytes.width, width(32));
        assert!(!unpacked_bytes.signed);
        assert_eq!(
            unpacked_bytes.shape,
            PortShape::UnpackedArray(UnpackedArrayShape {
                element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                    width: width(8),
                    signed: false,
                })),
                dimensions: vec![dimension(0, 3, 4)],
            })
        );

        let unpacked_bytes_out = find_port(&actual, "unpacked_bytes_out")?;
        assert_eq!(unpacked_bytes_out.direction, PortDirection::Output);
        assert_eq!(unpacked_bytes_out.width, width(32));
        assert!(!unpacked_bytes_out.signed);
        assert_eq!(
            unpacked_bytes_out.shape,
            PortShape::UnpackedArray(UnpackedArrayShape {
                element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                    width: width(8),
                    signed: false,
                })),
                dimensions: vec![dimension(0, 3, 4)],
            })
        );

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
            ports: vec![scalar_port("bus", PortDirection::Inout, 1, false)],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedInoutPort { port })
                if port == "bus"
        ));
    }

    #[test]
    fn accepts_wide_ports() -> Result<(), BuildError> {
        let metadata = DutMetadata {
            name: "dut".to_owned(),
            top_module: "dut".to_owned(),
            ports: vec![
                scalar_port("input_u65", PortDirection::Input, 65, false),
                scalar_port("input_i129", PortDirection::Input, 129, true),
                scalar_port("output_u256", PortDirection::Output, 256, false),
                scalar_port("output_i129", PortDirection::Output, 129, true),
            ],
        };

        validate_supported(&metadata)
    }

    #[test]
    fn rejects_aggregate_ports_after_normalization() -> Result<(), Box<dyn std::error::Error>> {
        let metadata = aggregate_ports_metadata()?;

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedPackedArrayPort { port }) if port == "packed_bytes"
        ));

        Ok(())
    }

    #[test]
    fn rejects_packed_array_port() {
        let metadata = DutMetadata {
            name: String::from("dut"),
            top_module: String::from("dut"),
            ports: vec![aggregate_port(
                "packed",
                PortDirection::Input,
                width(32),
                false,
                PortShape::PackedArray(PackedArrayShape {
                    element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                        width: width(8),
                        signed: false,
                    })),
                    dimensions: vec![dimension(3, 0, 4)],
                    width: width(32),
                    signed: false,
                }),
            )],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedPackedArrayPort { port }) if port == "packed"
        ));
    }

    #[test]
    fn rejects_packed_struct_port() {
        let metadata = DutMetadata {
            name: String::from("dut"),
            top_module: String::from("dut"),
            ports: vec![aggregate_port(
                "packet",
                PortDirection::Input,
                width(16),
                false,
                PortShape::PackedStruct(PackedStructShape {
                    width: width(16),
                    fields: vec![],
                    signed: false,
                }),
            )],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedPackedStructPort { port }) if port == "packet"
        ));
    }

    #[test]
    fn rejects_packed_enum_port() {
        let metadata = DutMetadata {
            name: String::from("dut"),
            top_module: String::from("dut"),
            ports: vec![aggregate_port(
                "state",
                PortDirection::Input,
                width(2),
                false,
                PortShape::PackedEnum(PackedEnumShape {
                    width: width(2),
                    signed: false,
                    variants: vec![],
                }),
            )],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedPackedEnumPort { port }) if port == "state"
        ));
    }

    #[test]
    fn rejects_unpacked_array_port() {
        let metadata = DutMetadata {
            name: String::from("dut"),
            top_module: String::from("dut"),
            ports: vec![aggregate_port(
                "bytes",
                PortDirection::Input,
                width(32),
                false,
                PortShape::UnpackedArray(UnpackedArrayShape {
                    element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                        width: width(8),
                        signed: false,
                    })),
                    dimensions: vec![dimension(0, 3, 4)],
                }),
            )],
        };

        assert!(matches!(
            validate_supported(&metadata),
            Err(BuildError::UnsupportedUnpackedArrayPort { port }) if port == "bytes"
        ));
    }
}
