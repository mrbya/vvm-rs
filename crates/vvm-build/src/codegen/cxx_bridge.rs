use super::names::DutNames;
use super::types::{PortType, UnpackedArrayType};
use crate::TraceOptions;
use crate::codegen::GENERATED_NOTICE;
use crate::metadata::{DutMetadata, PortDirection};
use crate::trace::TraceFormat;

/// Renders the raw CXX bridge for one DUT.
pub(super) fn render(
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
    timing: bool,
) -> String {
    let mut output = String::new();
    let traced = trace.is_some_and(|options| options.format == TraceFormat::Vcd);

    render_bridge_prelude(&mut output, names);
    render_bridge_lifecycle(&mut output, names, traced, timing);

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        render_port_operations(&mut output, port, port_names, names);
    }

    push_line(&mut output, "    }");
    push_line(&mut output, "}");

    output
}

/// Renders bridge operations for one normalized DUT port.
fn render_port_operations(
    output: &mut String,
    port: &crate::metadata::Port,
    port_names: &super::names::PortNames,
    names: &DutNames,
) {
    if let Some(array_type) = UnpackedArrayType::from_port(port) {
        let value_name = if matches!(
            array_type.element_type(),
            super::types::UnpackedArrayElementType::Wide(_)
        ) {
            "words"
        } else {
            "values"
        };
        let element_type = array_type.ffi_rust_element_type();
        push_line(output, "");
        match port.direction {
            PortDirection::Input => push_line(
                output,
                &format!("        /// Drives the `{}` DUT input.", port.name),
            ),
            PortDirection::Output => push_line(
                output,
                &format!("        /// Samples the `{}` DUT output.", port.name),
            ),
            PortDirection::Inout => return,
        }
        let signature = match port.direction {
            PortDirection::Input => format!(
                "        fn {}(self: Pin<&mut {}>, {value_name}: &[{element_type}]) -> bool;",
                port_names.method, names.cpp_type
            ),
            PortDirection::Output => format!(
                "        fn {}(self: &{}, {value_name}: &mut [{element_type}]) -> bool;",
                port_names.method, names.cpp_type
            ),
            PortDirection::Inout => return,
        };
        push_line(output, &signature);
        return;
    }
    let port_type = PortType::from_port(port);

    match port.direction {
        PortDirection::Input => {
            push_line(output, "");
            push_line(
                output,
                &format!("        /// Drives the `{}` DUT input.", port.name),
            );
            push_line(
                output,
                &match port_type {
                    PortType::Scalar(signal_type) => format!(
                        "        fn {}(self: Pin<&mut {}>, value: {});",
                        port_names.method,
                        names.cpp_type,
                        signal_type.rust_type()
                    ),
                    PortType::Wide(_) => format!(
                        "        fn {}(self: Pin<&mut {}>, words: &[u32]) -> bool;",
                        port_names.method, names.cpp_type
                    ),
                },
            );
        }
        PortDirection::Output => {
            push_line(output, "");
            push_line(
                output,
                &format!("        /// Samples the `{}` DUT output.", port.name),
            );
            push_line(
                output,
                &match port_type {
                    PortType::Scalar(signal_type) => format!(
                        "        fn {}(self: &{}) -> {};",
                        port_names.method,
                        names.cpp_type,
                        signal_type.rust_type()
                    ),
                    PortType::Wide(_) => format!(
                        "        fn {}(self: &{}, words: &mut [u32]) -> bool;",
                        port_names.method, names.cpp_type
                    ),
                },
            );
        }
        PortDirection::Inout => render_inout_operations(output, port, port_names, names, port_type),
    }
}

/// Renders the low-level input, output-enable, and output-value bridge operations for one inout.
fn render_inout_operations(
    output: &mut String,
    port: &crate::metadata::Port,
    port_names: &super::names::PortNames,
    names: &DutNames,
    port_type: PortType,
) {
    let Some(inout) = port_names.inout.as_ref() else {
        return;
    };
    let enable_type = PortType::from_port(&crate::metadata::Port {
        name: port.name.clone(),
        direction: port.direction,
        width: port.width,
        signed: false,
        shape: port.shape.clone(),
    });
    let signature = match port_type {
        PortType::Scalar(signal_type) => {
            format!(
                "        fn {}(self: Pin<&mut {}>, value: {});\n        fn {}(self: &{}) -> \
                     {};\n        fn {}(self: &{}) -> {};\n        fn {}(self: &{}) -> {};",
                inout.set_input,
                names.cpp_type,
                signal_type.rust_type(),
                inout.input,
                names.cpp_type,
                signal_type.rust_type(),
                inout.output_enable,
                names.cpp_type,
                enable_type.rust_value_type(),
                inout.output_value,
                names.cpp_type,
                signal_type.rust_type(),
            )
        }
        PortType::Wide(_) => format!(
            "        fn {}(self: Pin<&mut {}>, words: &[u32]) -> bool;\n        fn {}(self: \
                 &{}, words: &mut [u32]) -> bool;\n        fn {}(self: &{}, words: &mut [u32]) -> \
                 bool;\n        fn {}(self: &{}, words: &mut [u32]) -> bool;",
            inout.set_input,
            names.cpp_type,
            inout.input,
            names.cpp_type,
            inout.output_enable,
            names.cpp_type,
            inout.output_value,
            names.cpp_type,
        ),
    };

    push_line(output, "");
    push_line(
        output,
        &format!(
            "        /// Drives the externally supplied `{}` DUT input.",
            port.name
        ),
    );
    push_line(output, &signature);
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
fn render_bridge_lifecycle(output: &mut String, names: &DutNames, traced: bool, timing: bool) {
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
    if timing {
        push_line(output, "");
        push_line(
            output,
            "        /// Returns whether delayed HDL events remain pending.",
        );
        push_line(
            output,
            &format!(
                "        fn events_pending(self: &{}) -> bool;",
                names.cpp_type
            ),
        );
        push_line(output, "");
        push_line(
            output,
            "        /// Writes the absolute next delayed-event time.",
        );
        push_line(output, "        ///");
        push_line(
            output,
            "        /// Returns false when no event is pending.",
        );
        push_line(
            output,
            &format!(
                "        fn next_time_slot(self: &{}, time: &mut u64) -> bool;",
                names.cpp_type
            ),
        );
    }
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
