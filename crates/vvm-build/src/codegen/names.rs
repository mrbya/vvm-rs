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

    let factory = format!("create_{}", metadata.name);
    validate_cpp_identifier("C++ factory", &factory)?;

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
            PortDirection::Output => port.name.clone(),
            PortDirection::Inout => port.name.clone(),
        };

        validate_cpp_identifier("C++ adapter method", &method)?;

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

/// Converts an underscore-separated identifier to PascalCase.
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
