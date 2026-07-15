//! Deterministic generated C++ naming.

use std::collections::HashSet;

use crate::builder::validate_identifier;
use crate::codegen::types::{PackedArrayType, PackedStructType};
use crate::metadata::{DutMetadata, PortDirection};
use crate::{BuildError, BuildResult};

/// Resolved names for one generated DUT adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DutNames {
    /// Generated filename stem.
    pub file_stem: String,

    /// Nested namespace component below `vvm`.
    pub namespace: String,

    /// Generated C++ adapter class.
    pub cpp_type: String,

    /// Generated Rust error type.
    pub rust_error_type: String,

    /// Generated C++ factory function.
    pub factory: String,

    /// Verilator-generated model class.
    pub model_type: String,

    /// Resolved names for each port.
    pub ports: Vec<PortNames>,
}

/// Resolved names for one generated port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortNames {
    /// Verilator top-model accessor.
    pub accessor: String,

    /// Generated adapter member function.
    pub method: String,

    /// Generated safe Rust aggregate value type.
    pub rust_type: Option<String>,

    /// Generated packed-struct field methods.
    pub struct_fields: Vec<PackedStructFieldNames>,
}

/// Generated methods for one packed-struct field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedStructFieldNames {
    /// Generated field getter method.
    pub getter: String,

    /// Generated field setter method.
    pub setter: String,
}

/// Resolves and validates all names needed by the C++ adapter.
///
/// # Errors
///
/// Returns an error when a name cannot be represented safely or generated
/// operations would produce the same C++ member name.
pub fn resolve(metadata: &DutMetadata, model_prefix: &str) -> BuildResult<DutNames> {
    validate_cpp_identifier("DUT namespace", &metadata.name)?;

    validate_cpp_identifier("Verilator model type", model_prefix)?;

    let cpp_type = to_pascal_case(&metadata.name);

    if cpp_type.is_empty() {
        return Err(BuildError::UnsupportedCodegenName {
            role: "C++ adapter type",
            name: metadata.name.clone(),
            reason: "the transformed type name is empty",
        });
    }

    validate_cpp_identifier("C++ adapter type", &cpp_type)?;
    validate_rust_identifier("Rust DUT type", &cpp_type)?;

    let rust_error_type = format!("{cpp_type}Error");
    validate_rust_identifier("Rust DUT error type", &rust_error_type)?;

    let mut used_rust_types = HashSet::from([cpp_type.clone(), rust_error_type.clone()]);

    let factory = format!("create_{}", metadata.name);
    validate_cpp_identifier("C++ factory", &factory)?;
    validate_rust_identifier("Rust CXX factory", &factory)?;

    let mut used_members = HashSet::from([
        "eval".to_owned(),
        "finish".to_owned(),
        "impl_".to_owned(),
        cpp_type.clone(),
    ]);

    let mut ports = Vec::with_capacity(metadata.ports.len());

    for port in &metadata.ports {
        validate_cpp_identifier("HDL port accessor", &port.name)?;

        let method = match port.direction {
            PortDirection::Input => {
                format!("set_{}", port.name)
            }
            PortDirection::Output | PortDirection::Inout => port.name.clone(),
        };

        validate_cpp_identifier("C++ adapter method", &method)?;
        validate_rust_identifier("Rust DUT method", &method)?;

        if !used_members.insert(method.clone()) {
            return Err(BuildError::GeneratedNameCollision { name: method });
        }

        let rust_type = aggregate_rust_type(port)?;

        if let Some(ref rust_type) = rust_type {
            validate_rust_identifier("Rust packed aggregate type", rust_type)?;

            if !used_rust_types.insert(rust_type.clone()) {
                return Err(BuildError::GeneratedNameCollision {
                    name: rust_type.clone(),
                });
            }
        }

        let struct_fields = packed_struct_field_names(port)?;

        ports.push(PortNames {
            accessor: port.name.clone(),
            method,
            rust_type,
            struct_fields,
        });
    }

    Ok(DutNames {
        file_stem: metadata.name.clone(),
        namespace: metadata.name.clone(),
        cpp_type,
        rust_error_type,
        factory,
        model_type: model_prefix.to_owned(),
        ports,
    })
}

/// Returns the generated Rust wrapper type for one supported packed aggregate port.
fn aggregate_rust_type(port: &crate::metadata::Port) -> BuildResult<Option<String>> {
    let supported =
        PackedArrayType::from_port(port).is_some() || PackedStructType::from_port(port).is_some();

    if !supported {
        return Ok(None);
    }

    let rust_type = to_pascal_case(&port.name);

    if rust_type.is_empty() {
        return Err(BuildError::UnsupportedCodegenName {
            role: "Rust packed aggregate type",
            name: port.name.clone(),
            reason: "the transformed type name is empty",
        });
    }

    Ok(Some(rust_type))
}

/// Returns generated field method names for one supported packed struct port.
fn packed_struct_field_names(
    port: &crate::metadata::Port,
) -> BuildResult<Vec<PackedStructFieldNames>> {
    let Some(struct_type) = PackedStructType::from_port(port) else {
        return Ok(Vec::new());
    };

    let mut used_methods = HashSet::<String>::from([
        "zero".to_owned(),
        "from_bits".to_owned(),
        "bits".to_owned(),
        "into_bits".to_owned(),
        "from_words_le".to_owned(),
        "words_le".to_owned(),
        "layout".to_owned(),
        "fields".to_owned(),
        "width".to_owned(),
    ]);

    let mut names = Vec::with_capacity(struct_type.shape().fields.len());

    for field in struct_type.fields() {
        let getter = field.name().to_owned();
        let setter = format!("set_{}", field.name());

        validate_rust_identifier("Rust packed-struct field getter", &getter)?;
        validate_rust_identifier("Rust packed-struct field setter", &setter)?;

        if !used_methods.insert(getter.clone()) {
            return Err(BuildError::GeneratedNameCollision { name: getter });
        }

        if !used_methods.insert(setter.clone()) {
            return Err(BuildError::GeneratedNameCollision { name: setter });
        }

        names.push(PackedStructFieldNames { getter, setter });
    }

    Ok(names)
}

/// Verifies that a name is safe for direct C++ emission.
fn validate_cpp_identifier(role: &'static str, name: &str) -> BuildResult<()> {
    validate_identifier(role, name)?;

    if is_cpp_keyword(name) {
        return Err(BuildError::UnsupportedCodegenName {
            role,
            name: name.to_owned(),
            reason: "the name is a C++ keyword",
        });
    }

    if name.starts_with('_') || name.contains("__") {
        return Err(BuildError::UnsupportedCodegenName {
            role,
            name: name.to_owned(),
            reason: "reserved C++ identifier forms are not supported",
        });
    }

    Ok(())
}

/// Converts an underscore-separated identifier to `PascalCase`.
fn to_pascal_case(identifier: &str) -> String {
    let mut result = String::new();
    let mut uppercase_next = true;

    for character in identifier.chars() {
        if character == '_' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            result.push(character.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(character);
        }
    }

    result
}

/// Returns whether an identifier is a C++ keyword.
fn is_cpp_keyword(identifier: &str) -> bool {
    matches!(
        identifier,
        "alignas"
            | "alignof"
            | "and"
            | "and_eq"
            | "asm"
            | "auto"
            | "bitand"
            | "bitor"
            | "bool"
            | "break"
            | "case"
            | "catch"
            | "char"
            | "char8_t"
            | "char16_t"
            | "char32_t"
            | "class"
            | "compl"
            | "concept"
            | "const"
            | "consteval"
            | "constexpr"
            | "constinit"
            | "const_cast"
            | "continue"
            | "co_await"
            | "co_return"
            | "co_yield"
            | "decltype"
            | "default"
            | "delete"
            | "do"
            | "double"
            | "dynamic_cast"
            | "else"
            | "enum"
            | "explicit"
            | "export"
            | "extern"
            | "false"
            | "float"
            | "for"
            | "friend"
            | "goto"
            | "if"
            | "inline"
            | "int"
            | "long"
            | "mutable"
            | "namespace"
            | "new"
            | "noexcept"
            | "not"
            | "not_eq"
            | "nullptr"
            | "operator"
            | "or"
            | "or_eq"
            | "private"
            | "protected"
            | "public"
            | "register"
            | "reinterpret_cast"
            | "requires"
            | "return"
            | "short"
            | "signed"
            | "sizeof"
            | "static"
            | "static_assert"
            | "static_cast"
            | "struct"
            | "switch"
            | "template"
            | "this"
            | "thread_local"
            | "throw"
            | "true"
            | "try"
            | "typedef"
            | "typeid"
            | "typename"
            | "union"
            | "unsigned"
            | "using"
            | "virtual"
            | "void"
            | "volatile"
            | "wchar_t"
            | "while"
            | "xor"
            | "xor_eq"
    )
}

/// Verifies that a name is safe for direct Rust emission.
fn validate_rust_identifier(role: &'static str, name: &str) -> BuildResult<()> {
    validate_identifier(role, name)?;

    if is_rust_keyword(name) {
        return Err(BuildError::UnsupportedCodegenName {
            role,
            name: name.to_owned(),
            reason: "the name is a Rust keyword",
        });
    }

    Ok(())
}

/// Returns whether an identifier is a Rust keyword or reserved word.
fn is_rust_keyword(identifier: &str) -> bool {
    matches!(
        identifier,
        "Self"
            | "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "union"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::resolve;
    use crate::BuildError;
    use crate::metadata::{
        ArrayDimension, BitWidth, DutMetadata, PackedArrayShape, PackedScalarShape,
        PackedStructField, PackedStructShape, Port, PortDirection, PortShape,
    };

    fn port(name: &str, direction: PortDirection) -> Port {
        let width = BitWidth::new(NonZeroU32::MIN);

        Port {
            name: name.to_owned(),
            direction,
            width,
            signed: false,
            shape: PortShape::PackedScalar(PackedScalarShape {
                width,
                signed: false,
            }),
        }
    }

    fn packed_array_port(name: &str, direction: PortDirection) -> Port {
        let width = BitWidth::new(NonZeroU32::new(32).unwrap_or(NonZeroU32::MIN));
        let element_width = BitWidth::new(NonZeroU32::new(8).unwrap_or(NonZeroU32::MIN));

        Port {
            name: name.to_owned(),
            direction,
            width,
            signed: false,
            shape: PortShape::PackedArray(PackedArrayShape {
                element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                    width: element_width,
                    signed: false,
                })),
                dimensions: vec![ArrayDimension {
                    left: 3,
                    right: 0,
                    length: NonZeroU32::new(4).unwrap_or(NonZeroU32::MIN),
                }],
                width,
                signed: false,
            }),
        }
    }

    fn packed_struct_port(
        name: &str,
        direction: PortDirection,
        fields: Vec<PackedStructField>,
    ) -> Port {
        let width = BitWidth::new(NonZeroU32::new(32).unwrap_or(NonZeroU32::MIN));

        Port {
            name: name.to_owned(),
            direction,
            width,
            signed: false,
            shape: PortShape::PackedStruct(PackedStructShape {
                width,
                fields,
                signed: false,
            }),
        }
    }

    fn metadata(name: &str, ports: Vec<Port>) -> DutMetadata {
        DutMetadata {
            name: name.to_owned(),
            top_module: name.to_owned(),
            ports,
        }
    }

    #[test]
    fn resolves_counter_names() -> Result<(), BuildError> {
        let metadata = metadata(
            "counter",
            vec![
                port("clk", PortDirection::Input),
                port("count", PortDirection::Output),
            ],
        );

        let names = resolve(&metadata, "Vcounter")?;

        assert_eq!(names.file_stem, "counter");
        assert_eq!(names.namespace, "counter");
        assert_eq!(names.cpp_type, "Counter");
        assert_eq!(names.factory, "create_counter");
        assert_eq!(names.model_type, "Vcounter");

        assert_eq!(
            names.ports.first().map(|port| port.method.as_str()),
            Some("set_clk")
        );
        assert_eq!(
            names
                .ports
                .first()
                .and_then(|port| port.rust_type.as_deref()),
            None
        );

        assert_eq!(
            names.ports.get(1).map(|port| port.method.as_str()),
            Some("count")
        );
        assert_eq!(
            names
                .ports
                .get(1)
                .and_then(|port| port.rust_type.as_deref()),
            None
        );

        Ok(())
    }

    #[test]
    fn converts_snake_case_dut_name_to_pascal_case() -> Result<(), BuildError> {
        let metadata = metadata("pulse_counter", Vec::new());

        let names = resolve(&metadata, "Vpulse_counter")?;

        assert_eq!(names.cpp_type, "PulseCounter");
        assert_eq!(names.factory, "create_pulse_counter");

        Ok(())
    }

    #[test]
    fn accepts_keyword_port_when_generated_method_is_safe() -> Result<(), BuildError> {
        let metadata = metadata("dut", vec![port("match", PortDirection::Input)]);

        let names = resolve(&metadata, "Vdut")?;

        assert_eq!(
            names.ports.first().map(|port| port.method.as_str()),
            Some("set_match")
        );

        Ok(())
    }

    #[test]
    fn rejects_rust_keyword_output_method() {
        let metadata = metadata("dut", vec![port("match", PortDirection::Output)]);

        assert!(matches!(
            resolve(&metadata, "Vdut"),
            Err(BuildError::UnsupportedCodegenName {
                role: "Rust DUT method",
                name,
                ..
            }) if name == "match"
        ));
    }

    #[test]
    fn rejects_cpp_keyword_accessor() {
        let metadata = metadata("dut", vec![port("class", PortDirection::Input)]);

        assert!(matches!(
            resolve(&metadata, "Vdut"),
            Err(BuildError::UnsupportedCodegenName {
                role: "HDL port accessor",
                name,
                ..
            }) if name == "class"
        ));
    }

    #[test]
    fn rejects_transformed_method_collision() {
        let metadata = metadata(
            "dut",
            vec![
                port("value", PortDirection::Input),
                port("set_value", PortDirection::Output),
            ],
        );

        assert!(matches!(
            resolve(&metadata, "Vdut"),
            Err(BuildError::GeneratedNameCollision { name })
                if name == "set_value"
        ));
    }

    #[test]
    fn rejects_lifecycle_method_collision() {
        let metadata = metadata("dut", vec![port("eval", PortDirection::Output)]);

        assert!(matches!(
            resolve(&metadata, "Vdut"),
            Err(BuildError::GeneratedNameCollision { name })
                if name == "eval"
        ));
    }

    #[test]
    fn rejects_reserved_cpp_identifier_form() {
        let metadata = metadata("_dut", Vec::new());

        assert!(matches!(
            resolve(&metadata, "V_dut"),
            Err(BuildError::UnsupportedCodegenName {
                name,
                ..
            }) if name == "_dut"
        ));
    }

    #[test]
    fn resolves_packed_array_type_names() -> Result<(), BuildError> {
        let metadata = metadata(
            "packed_array_ports",
            vec![
                port("clk", PortDirection::Input),
                packed_array_port("packed_bytes", PortDirection::Input),
                packed_array_port("packed_bytes_out", PortDirection::Output),
            ],
        );

        let names = resolve(&metadata, "Vpacked_array_ports")?;

        assert_eq!(
            names
                .ports
                .get(1)
                .and_then(|port| port.rust_type.as_deref()),
            Some("PackedBytes")
        );
        assert_eq!(
            names
                .ports
                .get(2)
                .and_then(|port| port.rust_type.as_deref()),
            Some("PackedBytesOut")
        );

        Ok(())
    }

    #[test]
    fn rejects_packed_array_type_name_collision() {
        let metadata = metadata(
            "packed_bytes",
            vec![packed_array_port("packed_bytes", PortDirection::Input)],
        );

        assert!(matches!(
            resolve(&metadata, "Vdut"),
            Err(BuildError::GeneratedNameCollision { name }) if name == "PackedBytes"
        ));
    }

    #[test]
    fn resolves_packed_struct_type_and_field_names() -> Result<(), BuildError> {
        let field_width = BitWidth::new(NonZeroU32::new(4).unwrap_or(NonZeroU32::MIN));
        let bool_width = BitWidth::new(NonZeroU32::MIN);
        let scalar = |name: &str, offset: u32, width: BitWidth, signed: bool| PackedStructField {
            name: name.to_owned(),
            shape: PortShape::PackedScalar(PackedScalarShape { width, signed }),
            lsb_offset: offset,
            width,
            signed,
        };

        let metadata = metadata(
            "packed_struct_ports",
            vec![
                port("clk", PortDirection::Input),
                packed_struct_port(
                    "packet",
                    PortDirection::Input,
                    vec![
                        scalar("opcode", 28, field_width, false),
                        scalar("valid", 27, bool_width, false),
                        scalar("flags", 16, field_width, false),
                    ],
                ),
                packed_struct_port(
                    "packet_out",
                    PortDirection::Output,
                    vec![
                        scalar("opcode", 28, field_width, false),
                        scalar("valid", 27, bool_width, false),
                        scalar("flags", 16, field_width, false),
                    ],
                ),
            ],
        );

        let names = resolve(&metadata, "Vpacked_struct_ports")?;
        let packet = names
            .ports
            .get(1)
            .ok_or_else(|| BuildError::GeneratedNameCollision {
                name: String::from("missing packet port"),
            })?;
        let packet_out = names
            .ports
            .get(2)
            .ok_or_else(|| BuildError::GeneratedNameCollision {
                name: String::from("missing packet_out port"),
            })?;

        assert_eq!(packet.rust_type.as_deref(), Some("Packet"));
        assert_eq!(packet_out.rust_type.as_deref(), Some("PacketOut"));
        assert_eq!(
            packet
                .struct_fields
                .first()
                .map(|field| field.getter.as_str()),
            Some("opcode")
        );
        assert_eq!(
            packet
                .struct_fields
                .first()
                .map(|field| field.setter.as_str()),
            Some("set_opcode")
        );
        assert_eq!(
            packet
                .struct_fields
                .get(1)
                .map(|field| field.getter.as_str()),
            Some("valid")
        );
        assert_eq!(
            packet
                .struct_fields
                .get(1)
                .map(|field| field.setter.as_str()),
            Some("set_valid")
        );

        Ok(())
    }

    #[test]
    fn rejects_reserved_packed_struct_field_method_name() {
        let field_width = BitWidth::new(NonZeroU32::new(8).unwrap_or(NonZeroU32::MIN));
        let metadata = metadata(
            "packed_struct_ports",
            vec![packed_struct_port(
                "packet",
                PortDirection::Input,
                vec![PackedStructField {
                    name: String::from("bits"),
                    shape: PortShape::PackedScalar(PackedScalarShape {
                        width: field_width,
                        signed: false,
                    }),
                    lsb_offset: 0,
                    width: field_width,
                    signed: false,
                }],
            )],
        );

        assert!(matches!(
            resolve(&metadata, "Vpacked_struct_ports"),
            Err(BuildError::GeneratedNameCollision { name }) if name == "bits"
        ));
    }

    #[test]
    fn rejects_conflicting_packed_struct_field_methods() {
        let field_width = BitWidth::new(NonZeroU32::new(16).unwrap_or(NonZeroU32::MIN));
        let scalar = |name: &str, offset: u32| PackedStructField {
            name: name.to_owned(),
            shape: PortShape::PackedScalar(PackedScalarShape {
                width: field_width,
                signed: false,
            }),
            lsb_offset: offset,
            width: field_width,
            signed: false,
        };
        let metadata = metadata(
            "packed_struct_ports",
            vec![packed_struct_port(
                "packet",
                PortDirection::Input,
                vec![scalar("foo", 0), scalar("set_foo", 16)],
            )],
        );

        assert!(matches!(
            resolve(&metadata, "Vpacked_struct_ports"),
            Err(BuildError::GeneratedNameCollision { name }) if name == "set_foo"
        ));
    }
}
