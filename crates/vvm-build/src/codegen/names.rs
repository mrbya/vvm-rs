//! Deterministic generated C++ naming.

use std::collections::HashSet;

use crate::builder::validate_identifier;
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

        ports.push(PortNames {
            accessor: port.name.clone(),
            method,
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
        BitWidth, DutMetadata, PackedScalarShape, Port, PortDirection, PortShape,
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
            names.ports.get(1).map(|port| port.method.as_str()),
            Some("count")
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
}
