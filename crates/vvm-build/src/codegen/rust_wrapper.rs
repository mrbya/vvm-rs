use super::names::DutNames;
use super::types::SignalType;
use crate::metadata::{DutMetadata, Port, PortDirection};

/// Renders the safe Rust wrapper for one DUT.
pub(super) fn render(metadata: &DutMetadata, names: &DutNames) -> String {
    let mut output = String::new();

    push_line(&mut output, "include!(concat!(");
    push_line(&mut output, "    env!(\"OUT_DIR\"),");
    push_line(
        &mut output,
        &format!("    \"/vvm/{}/generated/bridge.rs\"", names.file_stem),
    );
    push_line(&mut output, "));");
    push_line(&mut output, "");

    push_line(
        &mut output,
        "/// Result type returned by generated DUT operations.",
    );
    push_line(
        &mut output,
        "pub type Result<T> = std::result::Result<T, &'static str>;",
    );
    push_line(&mut output, "");

    push_line(
        &mut output,
        &format!(
            "/// Safe Rust wrapper for the `{}` Verilated DUT.",
            metadata.top_module
        ),
    );
    push_line(&mut output, &format!("pub struct {} {{", names.cpp_type));
    push_line(
        &mut output,
        "    /// Opaque ownership of the generated C++ adapter.",
    );
    push_line(
        &mut output,
        &format!("    inner: cxx::UniquePtr<ffi::{}>,", names.cpp_type),
    );
    push_line(&mut output, "");
    push_line(
        &mut output,
        "    /// Tracks whether finalisation has already been forwarded.",
    );
    push_line(&mut output, "    finished: bool,");
    push_line(&mut output, "}");
    push_line(&mut output, "");

    push_line(&mut output, &format!("impl {} {{", names.cpp_type));

    render_constructor(&mut output, metadata, names);
    render_lifecycle(&mut output);

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        match port.direction {
            PortDirection::Input => {
                render_input(&mut output, port, &port_names.method);
            }
            PortDirection::Output => {
                render_output(&mut output, metadata, names, port, &port_names.method);
            }
            PortDirection::Inout => {}
        }
    }

    render_inner_mut(&mut output, names);

    push_line(&mut output, "}");
    push_line(&mut output, "");

    push_line(&mut output, &format!("impl Drop for {} {{", names.cpp_type));
    push_line(&mut output, "    fn drop(&mut self) {");
    push_line(&mut output, "        self.finish();");
    push_line(&mut output, "    }");
    push_line(&mut output, "}");

    output
}

/// Renders the safe DUT constructor.
fn render_constructor(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    push_line(output, "    /// Constructs the Verilated DUT.");
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error when the generated C++ adapter cannot be constructed.",
    );
    push_line(output, "    pub fn new() -> Result<Self> {");
    push_line(
        output,
        &format!("        let inner = ffi::{}();", names.factory),
    );
    push_line(output, "");
    push_line(output, "        if inner.is_null() {");
    push_line(
        output,
        &format!(
            "            return Err(\"failed to construct the Verilated {} model\");",
            metadata.name
        ),
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        Ok(Self {");
    push_line(output, "            inner,");
    push_line(output, "            finished: false,");
    push_line(output, "        })");
    push_line(output, "    }");
}

/// Renders evaluation and finalisation methods.
fn render_lifecycle(output: &mut String) {
    push_line(output, "");
    push_line(output, "    /// Evaluates the current DUT state.");
    push_line(output, "    pub fn eval(&mut self) {");
    push_line(output, "        self.inner_mut().eval();");
    push_line(output, "    }");

    push_line(output, "");
    push_line(output, "    /// Finalises the DUT exactly once.");
    push_line(output, "    pub fn finish(&mut self) {");
    push_line(output, "        if self.finished {");
    push_line(output, "            return;");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        self.inner_mut().finish();");
    push_line(output, "        self.finished = true;");
    push_line(output, "    }");
}

/// Renders one typed DUT input setter.
fn render_input(output: &mut String, port: &Port, method: &str) {
    let signal_type = SignalType::from_width(port.width);

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Drives the `{}` DUT input.", port.name),
    );
    push_line(
        output,
        &format!(
            "    pub fn {method}(&mut self, value: {}) {{",
            signal_type.rust_type()
        ),
    );
    push_line(
        output,
        &format!("        self.inner_mut().{method}(value);"),
    );
    push_line(output, "    }");
}

/// Renders one typed DUT output getter.
fn render_output(
    output: &mut String,
    metadata: &DutMetadata,
    names: &DutNames,
    port: &Port,
    method: &str,
) {
    let signal_type = SignalType::from_width(port.width);

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Samples the `{}` DUT output.", port.name),
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the internal C++ adapter pointer is unexpectedly null.",
    );
    push_line(
        output,
        &format!(
            "    pub fn {method}(&self) -> Result<{}> {{",
            signal_type.rust_type()
        ),
    );
    push_line(output, "        self.inner");
    push_line(output, "            .as_ref()");
    push_line(
        output,
        &format!("            .map(ffi::{}::{method})", names.cpp_type),
    );
    push_line(
        output,
        &format!(
            "            .ok_or(\"{} model unexpectedly became null\")",
            metadata.name
        ),
    );
    push_line(output, "    }");
}

/// Renders access to the pinned mutable CXX object.
fn render_inner_mut(output: &mut String, names: &DutNames) {
    push_line(output, "");
    push_line(
        output,
        "    /// Returns a pinned mutable reference to the generated C++ adapter.",
    );
    push_line(
        output,
        &format!(
            "    fn inner_mut(&mut self) -> std::pin::Pin<&mut ffi::{}> {{",
            names.cpp_type
        ),
    );
    push_line(output, "        self.inner.pin_mut()");
    push_line(output, "    }");
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
