//! Raw CXX bridge source generation.

use super::names::DutNames;
use super::types::SignalType;
use crate::metadata::{DutMetadata, PortDirection};

/// Renders the raw CXX bridge for one DUT.
pub(super) fn render(metadata: &DutMetadata, names: &DutNames) -> String {
    let mut output = String::new();

    push_line(
        &mut output,
        &format!("#[cxx::bridge(namespace = \"vvm::{}\")]", names.namespace),
    );
    push_line(
        &mut output,
        "/// Raw generated FFI bindings for the Verilated DUT adapter.",
    );
    push_line(&mut output, "mod ffi {");
    push_line(&mut output, "    unsafe extern \"C++\" {");
    push_line(
        &mut output,
        &format!("        include!(\"{}.hpp\");", names.file_stem),
    );
    push_line(&mut output, "");

    push_line(
        &mut output,
        "        /// Opaque generated Verilated DUT adapter.",
    );
    push_line(&mut output, &format!("        type {};", names.cpp_type));
    push_line(&mut output, "");

    push_line(
        &mut output,
        "        /// Constructs a Verilated DUT adapter.",
    );
    push_line(
        &mut output,
        &format!(
            "        fn {}() -> UniquePtr<{}>;",
            names.factory, names.cpp_type
        ),
    );
    push_line(&mut output, "");

    push_line(&mut output, "        /// Evaluates the current DUT state.");
    push_line(
        &mut output,
        &format!("        fn eval(self: Pin<&mut {}>);", names.cpp_type),
    );
    push_line(&mut output, "");

    push_line(&mut output, "        /// Finalises the DUT model.");
    push_line(
        &mut output,
        &format!("        fn finish(self: Pin<&mut {}>);", names.cpp_type),
    );

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        let signal_type = SignalType::from_width(port.width);

        match port.direction {
            PortDirection::Input => {
                push_line(&mut output, "");
                push_line(
                    &mut output,
                    &format!("        /// Drives the `{}` DUT input.", port.name),
                );
                push_line(
                    &mut output,
                    &format!(
                        "        fn {}(self: Pin<&mut {}>, value: {});",
                        port_names.method,
                        names.cpp_type,
                        signal_type.rust_type()
                    ),
                );
            }
            PortDirection::Output => {
                push_line(&mut output, "");
                push_line(
                    &mut output,
                    &format!("        /// Samples the `{}` DUT output.", port.name),
                );
                push_line(
                    &mut output,
                    &format!(
                        "        fn {}(self: &{}) -> {};",
                        port_names.method,
                        names.cpp_type,
                        signal_type.rust_type()
                    ),
                );
            }
            PortDirection::Inout => {}
        }
    }

    push_line(&mut output, "    }");
    push_line(&mut output, "}");

    output
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
