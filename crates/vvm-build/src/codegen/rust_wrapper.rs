//! Safe Rust DUT wrapper generation.

use super::GENERATED_NOTICE;
use super::names::DutNames;
use super::types::SignalType;
use crate::metadata::{DutMetadata, Port, PortDirection};

/// Renders the safe Rust wrapper for one DUT.
pub(super) fn render(metadata: &DutMetadata, names: &DutNames) -> String {
    let mut output = String::new();

    push_line(&mut output, GENERATED_NOTICE);
    push_line(&mut output, "");

    push_line(&mut output, "include!(concat!(");
    push_line(&mut output, "    env!(\"OUT_DIR\"),");
    push_line(
        &mut output,
        &format!("    \"/vvm/{}/generated/bridge.rs\"", names.file_stem),
    );
    push_line(&mut output, "));");
    push_line(&mut output, "");

    render_error(&mut output, metadata, names);
    push_line(&mut output, "");

    push_line(
        &mut output,
        "/// Result type returned by generated DUT operations.",
    );
    push_line(
        &mut output,
        &format!(
            "pub type Result<T> = std::result::Result<T, {}>;",
            names.rust_error_type
        ),
    );
    push_line(&mut output, "");

    render_struct(&mut output, metadata, names);
    push_line(&mut output, "");

    push_line(&mut output, "#[allow(dead_code)]");
    push_line(&mut output, &format!("impl {} {{", names.cpp_type));

    render_constructor(&mut output, metadata, names);
    render_lifecycle(&mut output, names);

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        match port.direction {
            PortDirection::Input => {
                render_input(&mut output, port, &port_names.method);
            }
            PortDirection::Output => {
                render_output(&mut output, port, &port_names.method);
            }
            PortDirection::Inout => {}
        }
    }

    render_internal_access(&mut output, names);

    push_line(&mut output, "}");
    push_line(&mut output, "");

    render_debug(&mut output, names);
    push_line(&mut output, "");

    render_dut_trait(&mut output, names);
    push_line(&mut output, "");

    render_drop(&mut output, names);

    output
}

/// Renders the generated DUT error type.
fn render_error(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    push_line(output, "/// Error returned by generated DUT operations.");
    push_line(output, "#[allow(dead_code)]");
    push_line(output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    push_line(output, &format!("pub enum {} {{", names.rust_error_type));
    push_line(
        output,
        "    /// The native Verilated model could not be constructed.",
    );
    push_line(output, "    ConstructionFailed,");
    push_line(output, "");
    push_line(
        output,
        "    /// The operation requires a DUT that has not been finished.",
    );
    push_line(output, "    Finished,");
    push_line(output, "");
    push_line(
        output,
        "    /// The internal native adapter is unexpectedly unavailable.",
    );
    push_line(output, "    AdapterUnavailable,");
    push_line(output, "}");
    push_line(output, "");

    push_line(
        output,
        &format!("impl std::fmt::Display for {} {{", names.rust_error_type),
    );
    push_line(output, "    fn fmt(");
    push_line(output, "        &self,");
    push_line(output, "        formatter: &mut std::fmt::Formatter<'_>,");
    push_line(output, "    ) -> std::fmt::Result {");
    push_line(output, "        let message = match *self {");
    push_line(
        output,
        &format!(
            "            Self::ConstructionFailed => \"failed to construct the Verilated {} \
             model\",",
            metadata.name
        ),
    );
    push_line(
        output,
        &format!(
            "            Self::Finished => \"the {} model has already been finished\",",
            metadata.name
        ),
    );
    push_line(
        output,
        &format!(
            "            Self::AdapterUnavailable => \"the {} native adapter is unexpectedly \
             unavailable\",",
            metadata.name
        ),
    );
    push_line(output, "        };");
    push_line(output, "");
    push_line(output, "        formatter.write_str(message)");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");

    push_line(
        output,
        &format!("impl std::error::Error for {} {{}}", names.rust_error_type),
    );
}

/// Renders the safe DUT structure.
fn render_struct(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    push_line(
        output,
        &format!(
            "/// Safe Rust wrapper for the `{}` Verilated DUT.",
            metadata.top_module
        ),
    );
    push_line(output, &format!("pub struct {} {{", names.cpp_type));
    push_line(
        output,
        "    /// Opaque ownership of the generated C++ adapter.",
    );
    push_line(
        output,
        &format!(
            "    inner: ::vvm::__private::cxx::UniquePtr<ffi::{}>,",
            names.cpp_type
        ),
    );
    push_line(output, "");
    push_line(
        output,
        "    /// Tracks whether finalisation has already completed.",
    );
    push_line(output, "    finished: bool,");
    push_line(output, "}");
}

/// Renders construction and pointer validation.
fn render_constructor(output: &mut String, _metadata: &DutMetadata, names: &DutNames) {
    push_line(output, "    /// Constructs the Verilated DUT.");
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error when the native adapter cannot be constructed.",
    );
    push_line(output, "    pub fn new() -> Result<Self> {");
    push_line(
        output,
        &format!("        let inner = ffi::{}();", names.factory),
    );
    push_line(output, "");
    push_line(output, "        Self::from_inner(inner)");
    push_line(output, "    }");
    push_line(output, "");

    push_line(
        output,
        "    /// Constructs Verilated DUT from an internal [`::vvm::__private::cxx::UniquePtr`].",
    );
    push_line(
        output,
        &format!(
            "    fn from_inner(inner: ::vvm::__private::cxx::UniquePtr<ffi::{}>) -> Result<Self> \
             {{",
            names.cpp_type
        ),
    );
    push_line(output, "        if inner.is_null() {");
    push_line(
        output,
        &format!(
            "            return Err({}::ConstructionFailed);",
            names.rust_error_type
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

/// Renders lifecycle operations.
fn render_lifecycle(output: &mut String, names: &DutNames) {
    push_line(output, "");
    push_line(
        output,
        "    /// Returns whether the DUT has been finalised.",
    );
    push_line(output, "    #[must_use]");
    push_line(output, "    pub const fn is_finished(&self) -> bool {");
    push_line(output, "        self.finished");
    push_line(output, "    }");

    push_line(output, "");
    push_line(output, "    /// Evaluates the current DUT state.");
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT has already been finished.",
    );
    push_line(output, "    pub fn eval(&mut self) -> Result<()> {");
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "        self.inner_mut()?.eval();");
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    push_line(output, "");
    push_line(output, "    /// Finalises the DUT exactly once.");
    push_line(output, "    ///");
    push_line(output, "    /// Calling this method repeatedly is safe.");
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the native adapter is unexpectedly unavailable.",
    );
    push_line(output, "    pub fn finish(&mut self) -> Result<()> {");
    push_line(output, "        if self.finished {");
    push_line(output, "            return Ok(());");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        self.inner_mut()?.finish();");
    push_line(output, "        self.finished = true;");
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    let _ = names;
}

/// Renders one typed input setter.
fn render_input(output: &mut String, port: &Port, method: &str) {
    let signal_type = SignalType::from_width(port.width);

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Drives the `{}` DUT input.", port.name),
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT has already been finished.",
    );
    push_line(
        output,
        &format!(
            "    pub fn {method}(&mut self, value: {}) -> Result<()> {{",
            signal_type.rust_type()
        ),
    );
    push_line(output, "        self.ensure_running()?;");
    push_line(
        output,
        &format!("        self.inner_mut()?.{method}(value);"),
    );
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
}

/// Renders one typed output getter.
fn render_output(output: &mut String, port: &Port, method: &str) {
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
        "    /// Returns an error if the DUT has already been finished.",
    );
    push_line(
        output,
        &format!(
            "    pub fn {method}(&self) -> Result<{}> {{",
            signal_type.rust_type()
        ),
    );
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");
    push_line(output, &format!("        Ok(self.inner_ref()?.{method}())"));
    push_line(output, "    }");
}

/// Renders checked internal native access.
fn render_internal_access(output: &mut String, names: &DutNames) {
    push_line(output, "");
    push_line(
        output,
        "    /// Ensures that operations may still access the DUT.",
    );
    push_line(output, "    const fn ensure_running(&self) -> Result<()> {");
    push_line(output, "        if self.finished {");
    push_line(
        output,
        &format!(
            "            return Err({}::Finished);",
            names.rust_error_type
        ),
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    push_line(output, "");
    push_line(
        output,
        "    /// Returns an immutable reference to the native adapter.",
    );
    push_line(
        output,
        &format!(
            "    fn inner_ref(&self) -> Result<&ffi::{}> {{",
            names.cpp_type
        ),
    );
    push_line(output, "        self.inner.as_ref().ok_or(");
    push_line(
        output,
        &format!("            {}::AdapterUnavailable,", names.rust_error_type),
    );
    push_line(output, "        )");
    push_line(output, "    }");

    push_line(output, "");
    push_line(
        output,
        "    /// Returns a pinned mutable reference to the native adapter.",
    );
    push_line(
        output,
        &format!(
            "    fn inner_mut(&mut self) -> Result<std::pin::Pin<&mut ffi::{}>> {{",
            names.cpp_type
        ),
    );
    push_line(output, "        self.inner.as_mut().ok_or(");
    push_line(
        output,
        &format!("            {}::AdapterUnavailable,", names.rust_error_type),
    );
    push_line(output, "        )");
    push_line(output, "    }");
}

/// Renders the safe debug implementation.
fn render_debug(output: &mut String, names: &DutNames) {
    push_line(
        output,
        &format!("impl std::fmt::Debug for {} {{", names.cpp_type),
    );
    push_line(output, "    fn fmt(");
    push_line(output, "        &self,");
    push_line(output, "        formatter: &mut std::fmt::Formatter<'_>,");
    push_line(output, "    ) -> std::fmt::Result {");
    push_line(
        output,
        &format!("        formatter.debug_struct(\"{}\")", names.cpp_type),
    );
    push_line(output, "            .field(\"finished\", &self.finished)");
    push_line(output, "            .finish_non_exhaustive()");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Renders the VVM DUT trait implementation.
fn render_dut_trait(output: &mut String, names: &DutNames) {
    push_line(
        output,
        &format!("impl ::vvm::Dut for {} {{", names.cpp_type),
    );
    push_line(
        output,
        &format!("    type Error = {};", names.rust_error_type),
    );
    push_line(output, "");

    push_line(output, "    fn evaluate(&mut self) -> Result<()> {");
    push_line(output, "        Self::eval(self)");
    push_line(output, "    }");
    push_line(output, "");

    push_line(output, "    fn finalize(&mut self) -> Result<()> {");
    push_line(output, "        Self::finish(self)");
    push_line(output, "    }");

    push_line(output, "}");
}

/// Renders safe drop-time finalisation.
fn render_drop(output: &mut String, names: &DutNames) {
    push_line(output, &format!("impl Drop for {} {{", names.cpp_type));
    push_line(output, "    fn drop(&mut self) {");
    push_line(output, "        if self.finished {");
    push_line(output, "            return;");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        if let Some(inner) = self.inner.as_mut() {");
    push_line(output, "            inner.finish();");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        self.finished = true;");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
