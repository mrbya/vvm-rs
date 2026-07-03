use super::names::DutNames;
use super::types::SignalType;
use crate::TraceOptions;
use crate::codegen::GENERATED_NOTICE;
use crate::metadata::{DutMetadata, PortDirection};
use crate::trace::TraceFormat;

/// Renders the raw CXX bridge for one DUT.
pub(super) fn render(
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
) -> String {
    let mut output = String::new();
    let traced = trace.is_some_and(|options| options.format == TraceFormat::Vcd);

    render_bridge_prelude(&mut output, names);
    render_bridge_lifecycle(&mut output, names, traced);

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        let signal_type = SignalType::from_port(port);

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

/// Renders the bridge prelude and type declarations.
fn render_bridge_prelude(output: &mut String, names: &DutNames) {
    push_line(output, GENERATED_NOTICE);
    push_line(output, "");
    push_line(
        output,
        &format!("#[cxx::bridge(namespace = \"vvm::{}\")]", names.namespace),
    );
    push_line(
        output,
        "/// Raw generated FFI bindings for the Verilated DUT adapter.",
    );
    push_line(output, "mod ffi {");
    push_line(output, "    unsafe extern \"C++\" {");
    push_line(
        output,
        &format!("        include!(\"{}.hpp\");", names.file_stem),
    );
    push_line(output, "");
    push_line(
        output,
        "        /// Opaque generated Verilated DUT adapter.",
    );
    push_line(output, &format!("        type {};", names.cpp_type));
    push_line(output, "");
    push_line(output, "        /// Constructs a Verilated DUT adapter.");
    push_line(
        output,
        &format!(
            "        fn {}() -> UniquePtr<{}>;",
            names.factory, names.cpp_type
        ),
    );
    push_line(output, "");
}

/// Renders bridge lifecycle and optional trace methods.
fn render_bridge_lifecycle(output: &mut String, names: &DutNames, traced: bool) {
    push_line(output, "        /// Evaluates the current DUT state.");
    push_line(
        output,
        &format!("        fn eval(self: Pin<&mut {}>);", names.cpp_type),
    );
    push_line(output, "");
    push_line(
        output,
        "        /// Advances the native simulation context time.",
    );
    push_line(
        output,
        &format!(
            "        fn advance_time(self: Pin<&mut {}>, delta: u64) -> bool;",
            names.cpp_type
        ),
    );
    push_line(output, "");
    push_line(output, "        /// Finalises the DUT model.");
    push_line(
        output,
        &format!("        fn finish(self: Pin<&mut {}>);", names.cpp_type),
    );
    if traced {
        push_line(output, "");
        push_line(
            output,
            "        /// Opens the generated VCD waveform trace.",
        );
        push_line(
            output,
            &format!(
                "        fn open_trace(self: Pin<&mut {}>, path: &str) -> bool;",
                names.cpp_type
            ),
        );
        push_line(output, "");
        push_line(output, "        /// Flushes and closes the waveform trace.");
        push_line(
            output,
            &format!(
                "        fn close_trace(self: Pin<&mut {}>);",
                names.cpp_type
            ),
        );
        push_line(output, "");
        push_line(
            output,
            "        /// Returns whether the waveform trace is open.",
        );
        push_line(
            output,
            &format!(
                "        fn trace_is_open(self: &{}) -> bool;",
                names.cpp_type
            ),
        );
    }
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
