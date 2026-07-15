//! Safe Rust DUT wrapper generation.

use super::GENERATED_NOTICE;
use super::names::{DutNames, PortNames};
use super::types::{
    PackedArrayType, PortType, SignalType, WideType, contains_packed_array_ports,
    contains_wide_ports,
};
use crate::TraceOptions;
use crate::metadata::{DutMetadata, Port, PortDirection};
use crate::trace::TraceFormat;

/// Renders the safe Rust wrapper for one DUT.
pub(super) fn render(
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
) -> String {
    let mut output = String::new();
    let traced = trace.is_some_and(|options| options.format == TraceFormat::Vcd);

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

    render_packed_array_types(&mut output, metadata, names);

    if traced {
        render_trace_flag(&mut output);
        push_line(&mut output, "");
    }

    render_error(&mut output, metadata, names, traced);
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

    render_struct(&mut output, metadata, names, traced);
    push_line(&mut output, "");

    push_line(&mut output, "#[allow(dead_code)]");
    if traced {
        push_line(&mut output, "#[allow(clippy::same_name_method)]");
    }
    push_line(&mut output, &format!("impl {} {{", names.cpp_type));

    render_constructor(&mut output, metadata, names, traced);
    render_lifecycle(&mut output, names, traced);
    render_timing(&mut output, names);

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        match port.direction {
            PortDirection::Input => {
                render_input(&mut output, port, port_names, names);
            }
            PortDirection::Output => {
                render_output(&mut output, port, port_names, names);
            }
            PortDirection::Inout => {}
        }
    }

    render_internal_access(&mut output, names);

    push_line(&mut output, "}");
    push_line(&mut output, "");

    render_debug(&mut output, names, traced);
    push_line(&mut output, "");

    render_dut_trait(&mut output, names);
    push_line(&mut output, "");

    if traced {
        render_traceable_trait(&mut output, names);
        push_line(&mut output, "");
    }

    render_drop(&mut output, names, traced);

    output
}

/// Renders generated packed-array wrapper types.
fn render_packed_array_types(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    let mut rendered_any = false;

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        let Some(array_type) = PackedArrayType::from_port(port) else {
            continue;
        };

        let Some(rust_type) = port_names.rust_type.as_deref() else {
            continue;
        };

        if rendered_any {
            push_line(output, "");
        }

        render_packed_array_type(output, port, rust_type, array_type);
        rendered_any = true;
    }

    if rendered_any {
        push_line(output, "");
    }
}

/// Renders one generated packed-array wrapper type.
#[allow(clippy::too_many_lines)]
fn render_packed_array_type(
    output: &mut String,
    port: &Port,
    rust_type: &str,
    array_type: PackedArrayType<'_>,
) {
    let storage_type = array_type.storage_rust_type();
    let storage_constructor_type = array_type.storage_constructor_type();
    let element_rust_type = array_type.element_rust_type();
    let extraction_function = array_type.extraction_function();
    let insertion_function = array_type.insertion_function();
    let total_width = array_type.total_width();
    let element_width = array_type.element_width();
    let length = array_type.length();
    let left = array_type.left();
    let right = array_type.right();

    push_line(
        output,
        &format!(
            "/// Packed-array value used by the `{}` DUT port.",
            port.name
        ),
    );
    push_line(output, "#[derive(Debug, Clone, PartialEq, Eq, Hash)]");
    push_line(output, &format!("pub struct {rust_type} {{"));
    push_line(output, "    /// Canonical flattened packed storage.");
    push_line(output, &format!("    bits: {storage_type},"));
    push_line(output, "}");
    push_line(output, "");
    push_line(output, "#[allow(clippy::same_name_method)]");
    push_line(output, &format!("impl {rust_type} {{"));
    push_line(output, "    /// Total packed width.");
    push_line(
        output,
        &format!("    pub const WIDTH: usize = {total_width};"),
    );
    push_line(output, "");
    push_line(output, "    /// Number of canonical transfer words.");
    push_line(
        output,
        &format!("    pub const WORDS: usize = {storage_constructor_type}::WORDS;"),
    );
    push_line(output, "");
    push_line(output, "    /// Number of packed-array elements.");
    push_line(output, &format!("    pub const LEN: usize = {length};"));
    push_line(output, "");
    push_line(output, "    /// Width of one element.");
    push_line(
        output,
        &format!("    pub const ELEMENT_WIDTH: usize = {element_width};"),
    );
    push_line(output, "");
    push_line(output, "    /// Left HDL index bound.");
    push_line(output, &format!("    pub const LEFT: i64 = {left};"));
    push_line(output, "");
    push_line(output, "    /// Right HDL index bound.");
    push_line(output, &format!("    pub const RIGHT: i64 = {right};"));
    push_line(output, "");
    push_line(output, "    /// Constructs an all-zero packed-array value.");
    push_line(output, "    #[must_use]");
    push_line(output, "    pub fn zero() -> Self {");
    push_line(output, "        Self {");
    push_line(
        output,
        &format!("            bits: {storage_constructor_type}::zero(),"),
    );
    push_line(output, "        }");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    /// Wraps canonical packed bits.");
    push_line(output, "    #[must_use]");
    push_line(
        output,
        &format!("    pub const fn from_bits(bits: {storage_type}) -> Self {{"),
    );
    push_line(output, "        Self {");
    push_line(output, "            bits,");
    push_line(output, "        }");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    /// Returns the underlying packed bits.");
    push_line(output, "    #[must_use]");
    push_line(
        output,
        &format!("    pub const fn bits(&self) -> &{storage_type} {{"),
    );
    push_line(output, "        &self.bits");
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Consumes the value and returns its packed bits.",
    );
    push_line(output, "    #[must_use]");
    push_line(
        output,
        &format!("    pub fn into_bits(self) -> {storage_type} {{"),
    );
    push_line(output, "        self.bits");
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Constructs the packed array from canonical words.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error when the word count does not match the packed width.",
    );
    push_line(output, "    pub fn from_words_le(");
    push_line(output, "        words: impl AsRef<[u32]>,");
    push_line(
        output,
        "    ) -> std::result::Result<Self, ::vvm::InvalidBitVectorWordCount> {",
    );
    push_line(
        output,
        &format!("        {storage_constructor_type}::from_words_le(words)"),
    );
    push_line(output, "            .map(Self::from_bits)");
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Returns canonical least-significant-word-first words.",
    );
    push_line(output, "    #[must_use]");
    push_line(output, "    pub fn words_le(&self) -> &[u32] {");
    push_line(output, "        self.bits.words_le()");
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Computes the packed-bit offset for one HDL element index.",
    );
    push_line(output, "    fn element_offset(");
    push_line(output, "        index: i64,");
    push_line(
        output,
        "    ) -> std::result::Result<usize, ::vvm::PackedLayoutError> {",
    );
    push_line(
        output,
        "        let (lower, upper) = if Self::LEFT <= Self::RIGHT {",
    );
    push_line(output, "            (Self::LEFT, Self::RIGHT)");
    push_line(output, "        } else {");
    push_line(output, "            (Self::RIGHT, Self::LEFT)");
    push_line(output, "        };");
    push_line(output, "");
    push_line(
        output,
        "        let in_range = (lower..=upper).contains(&index);",
    );
    push_line(output, "");
    push_line(output, "        if !in_range {");
    push_line(
        output,
        "            return Err(::vvm::PackedLayoutError::range_out_of_bounds(",
    );
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::ELEMENT_WIDTH,");
    push_line(output, "            ));");
    push_line(output, "        }");
    push_line(output, "");
    push_line(
        output,
        "        let ordinal = if Self::LEFT >= Self::RIGHT {",
    );
    push_line(output, "            index.checked_sub(Self::RIGHT)");
    push_line(output, "        } else {");
    push_line(output, "            Self::RIGHT.checked_sub(index)");
    push_line(output, "        };");
    push_line(output, "");
    push_line(output, "        let Some(ordinal) = ordinal else {");
    push_line(
        output,
        "            return Err(::vvm::PackedLayoutError::range_out_of_bounds(",
    );
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::ELEMENT_WIDTH,");
    push_line(output, "            ));");
    push_line(output, "        };");
    push_line(output, "");
    push_line(
        output,
        "        let ordinal = usize::try_from(ordinal).map_err(|_error| {",
    );
    push_line(
        output,
        "            ::vvm::PackedLayoutError::range_out_of_bounds(",
    );
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::ELEMENT_WIDTH,");
    push_line(output, "            )");
    push_line(output, "        })?;");
    push_line(output, "");
    push_line(
        output,
        "        ordinal.checked_mul(Self::ELEMENT_WIDTH).ok_or(",
    );
    push_line(
        output,
        "            ::vvm::PackedLayoutError::range_out_of_bounds(",
    );
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::WIDTH,");
    push_line(output, "                Self::ELEMENT_WIDTH,");
    push_line(output, "            ),");
    push_line(output, "        )");
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Returns one packed-array element using its HDL index.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error when the HDL index is outside the declared range or packed",
    );
    push_line(output, "    /// layout extraction fails.");
    push_line(
        output,
        &format!(
            "    pub fn element(&self, index: i64) -> std::result::Result<{element_rust_type}, \
             ::vvm::PackedLayoutError> {{"
        ),
    );
    push_line(output, "        let offset = Self::element_offset(index)?;");
    push_line(output, "");
    if array_type.element_is_bool() {
        push_line(output, "        Ok(");
        push_line(output, &format!("            {extraction_function}("));
        push_line(output, "                self.words_le(),");
        push_line(output, "                Self::WIDTH,");
        push_line(output, "                offset,");
        push_line(output, "                1,");
        push_line(output, "            )? != 0,");
        push_line(output, "        )");
    } else {
        push_line(output, &format!("        let raw = {extraction_function}("));
        push_line(output, "            self.words_le(),");
        push_line(output, "            Self::WIDTH,");
        push_line(output, "            offset,");
        push_line(output, "            Self::ELEMENT_WIDTH,");
        push_line(output, "        )?;");
        push_line(output, "");
        push_line(
            output,
            &format!("        {element_rust_type}::try_from(raw).map_err(|_error| {{"),
        );
        push_line(
            output,
            "            ::vvm::PackedLayoutError::range_out_of_bounds(",
        );
        push_line(output, "                Self::WIDTH,");
        push_line(output, "                offset,");
        push_line(output, "                Self::ELEMENT_WIDTH,");
        push_line(output, "            )");
        push_line(output, "        })");
    }
    push_line(output, "    }");
    push_line(output, "");
    push_line(
        output,
        "    /// Updates one packed-array element using its HDL index.",
    );
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Values wider than the declared element width are truncated to the low bits.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error when the HDL index is outside the declared range or packed",
    );
    push_line(output, "    /// layout insertion fails.");
    push_line(
        output,
        &format!(
            "    pub fn set_element(&mut self, index: i64, value: {element_rust_type}) -> \
             std::result::Result<(), ::vvm::PackedLayoutError> {{"
        ),
    );
    push_line(output, "        let offset = Self::element_offset(index)?;");
    push_line(output, "        let mut words = self.words_le().to_vec();");
    push_line(output, "");
    push_line(output, &format!("        {insertion_function}("));
    push_line(output, "            &mut words,");
    push_line(output, "            Self::WIDTH,");
    push_line(output, "            offset,");
    push_line(output, "            Self::ELEMENT_WIDTH,");
    if array_type.element_is_bool() {
        push_line(output, "            u64::from(u8::from(value)),");
    } else if array_type.element_signed() {
        push_line(output, "            i64::from(value),");
    } else {
        push_line(output, "            u64::from(value),");
    }
    push_line(output, "        )?;");
    push_line(output, "");
    push_line(
        output,
        &format!("        self.bits = {storage_constructor_type}::from_words_le(words)"),
    );
    push_line(
        output,
        "            .map_err(::vvm::PackedLayoutError::invalid_packed_value)?;",
    );
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
    push_line(output, &format!("impl Default for {rust_type} {{"));
    push_line(output, "    fn default() -> Self {");
    push_line(output, "        Self::zero()");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
    push_line(output, "#[allow(clippy::same_name_method)]");
    push_line(
        output,
        &format!("impl ::vvm::PackedValue for {rust_type} {{"),
    );
    push_line(output, "    const WIDTH: usize = Self::WIDTH;");
    push_line(output, "    const WORDS: usize = Self::WORDS;");
    push_line(output, "");
    push_line(output, "    fn from_words_le(");
    push_line(output, "        words: impl AsRef<[u32]>,");
    push_line(
        output,
        "    ) -> std::result::Result<Self, ::vvm::InvalidBitVectorWordCount> {",
    );
    push_line(output, "        Self::from_words_le(words)");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    fn words_le(&self) -> &[u32] {");
    push_line(output, "        self.words_le()");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
    push_line(
        output,
        &format!("impl From<{storage_type}> for {rust_type} {{"),
    );
    push_line(
        output,
        &format!("    fn from(bits: {storage_type}) -> Self {{"),
    );
    push_line(output, "        Self::from_bits(bits)");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
    push_line(
        output,
        &format!("impl From<{rust_type}> for {storage_type} {{"),
    );
    push_line(
        output,
        &format!("    fn from(value: {rust_type}) -> Self {{"),
    );
    push_line(output, "        value.into_bits()");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Renders the generated DUT error type.
fn render_error(output: &mut String, metadata: &DutMetadata, names: &DutNames, traced: bool) {
    push_line(output, "/// Error returned by generated DUT operations.");
    push_line(output, "#[allow(dead_code)]");
    push_line(output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    push_line(output, &format!("pub enum {} {{", names.rust_error_type));
    render_error_variants(output, metadata, traced);
    push_line(output, "}");
    push_line(output, "");

    render_error_display(output, metadata, names, traced);
    push_line(output, "");

    push_line(
        output,
        &format!("impl std::error::Error for {} {{}}", names.rust_error_type),
    );
}

/// Renders the generated DUT error variants.
fn render_error_variants(output: &mut String, metadata: &DutMetadata, traced: bool) {
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
    push_line(output, "");
    push_line(output, "    /// Simulation time would overflow.");
    push_line(output, "    TimeOverflow,");
    if contains_packed_array_ports(metadata) {
        push_line(output, "");
        push_line(
            output,
            "    /// A generated packed-array layout conversion failed.",
        );
        push_line(output, "    PackedLayoutFailed,");
    }
    if contains_wide_ports(metadata) {
        push_line(output, "");
        push_line(
            output,
            "    /// The generated wide-port transfer was rejected by the native adapter.",
        );
        push_line(output, "    WidePortTransferFailed,");
    }
    if traced {
        push_line(output, "");
        push_line(
            output,
            "    /// Trace configuration was requested after evaluation began.",
        );
        push_line(output, "    TraceAfterEvaluation,");
        push_line(output, "");
        push_line(
            output,
            "    /// A waveform trace has already been configured.",
        );
        push_line(output, "    TraceAlreadyConfigured,");
        push_line(output, "");
        push_line(
            output,
            "    /// The waveform trace path is not valid UTF-8.",
        );
        push_line(output, "    TracePathNotUtf8,");
        push_line(output, "");
        push_line(
            output,
            "    /// The native waveform trace could not be opened.",
        );
        push_line(output, "    TraceOpenFailed,");
    }
}

/// Renders the generated DUT error display implementation.
fn render_error_display(
    output: &mut String,
    metadata: &DutMetadata,
    names: &DutNames,
    traced: bool,
) {
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
    push_line(
        output,
        &format!(
            "            Self::TimeOverflow => \"advancing {} simulation time would overflow\",",
            metadata.name
        ),
    );
    if contains_packed_array_ports(metadata) {
        push_line(
            output,
            "            Self::PackedLayoutFailed => {\"the generated packed-array layout \
             conversion failed\"},",
        );
    }
    if contains_wide_ports(metadata) {
        push_line(
            output,
            "            Self::WidePortTransferFailed => {\"the native adapter rejected a \
             wide-port transfer\"},",
        );
    }
    if traced {
        push_line(
            output,
            "            Self::TraceAfterEvaluation => {\"waveform tracing must be configured \
             before the first evaluation\"},",
        );
        push_line(
            output,
            "            Self::TraceAlreadyConfigured => {\"waveform tracing has already been \
             configured for this DUT\"},",
        );
        push_line(
            output,
            "            Self::TracePathNotUtf8 => {\"waveform trace path is not valid UTF-8\"},",
        );
        push_line(
            output,
            "            Self::TraceOpenFailed => {\"failed to open the native VCD waveform \
             trace\"}",
        );
    }
    push_line(output, "        };");
    push_line(output, "");
    push_line(output, "        formatter.write_str(message)");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Renders a small bool wrapper to keep traced state explicit.
fn render_trace_flag(output: &mut String) {
    push_line(output, "/// Compact generated trace-state flag.");
    push_line(output, "#[derive(Clone, Copy, Default, PartialEq, Eq)]");
    push_line(output, "struct TraceFlag(bool);");
    push_line(output, "");
    push_line(output, "impl TraceFlag {");
    push_line(output, "    /// Creates a flag from a raw boolean state.");
    push_line(output, "    const fn new(value: bool) -> Self {");
    push_line(output, "        Self(value)");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    /// Returns whether the flag is currently set.");
    push_line(output, "    const fn is_set(self) -> bool {");
    push_line(output, "        self.0");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
    push_line(output, "impl std::fmt::Debug for TraceFlag {");
    push_line(output, "    fn fmt(");
    push_line(output, "        &self,");
    push_line(output, "        formatter: &mut std::fmt::Formatter<'_>,");
    push_line(output, "    ) -> std::fmt::Result {");
    push_line(output, "        self.0.fmt(formatter)");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Renders the safe DUT structure.
fn render_struct(output: &mut String, metadata: &DutMetadata, names: &DutNames, traced: bool) {
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
    push_line(output, "");
    push_line(output, "    /// Current logical simulation time.");
    push_line(output, "    time: ::vvm::SimulationTime,");
    if traced {
        push_line(output, "");
        push_line(output, "    /// Whether the DUT has been evaluated.");
        push_line(output, "    evaluated: TraceFlag,");
        push_line(output, "");
        push_line(
            output,
            "    /// Whether trace configuration has been attempted.",
        );
        push_line(output, "    trace_configured: TraceFlag,");
        push_line(output, "");
        push_line(
            output,
            "    /// Whether the native trace is currently open.",
        );
        push_line(output, "    trace_open: TraceFlag,");
    }
    push_line(output, "}");
}

/// Renders construction and pointer validation.
fn render_constructor(
    output: &mut String,
    _metadata: &DutMetadata,
    names: &DutNames,
    traced: bool,
) {
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
    push_line(output, "            time: ::vvm::SimulationTime::ZERO,");
    if traced {
        push_line(output, "            evaluated: TraceFlag::new(false),");
        push_line(
            output,
            "            trace_configured: TraceFlag::new(false),",
        );
        push_line(output, "            trace_open: TraceFlag::new(false),");
    }
    push_line(output, "        })");
    push_line(output, "    }");
}

/// Renders lifecycle operations.
fn render_lifecycle(output: &mut String, names: &DutNames, traced: bool) {
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
    if traced {
        push_line(output, "        self.evaluated = TraceFlag::new(true);");
    }
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    if traced {
        render_trace_lifecycle(output, names);
    }

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
    if traced {
        push_line(output, "        self.trace_open = TraceFlag::new(false);");
    }
    push_line(output, "        self.finished = true;");
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    let _ = names;
}

/// Renders optional trace lifecycle methods.
fn render_trace_lifecycle(output: &mut String, names: &DutNames) {
    render_open_trace_method(output, names);
    render_close_trace_methods(output);
}

/// Renders the safe trace opening method.
fn render_open_trace_method(output: &mut String, names: &DutNames) {
    push_line(output, "");
    push_line(output, "    /// Opens the generated VCD waveform trace.");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// This must be called before the first DUT evaluation.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT is finished, has already been evaluated,",
    );
    push_line(
        output,
        "    /// tracing was previously configured, the path is not valid UTF-8, or the",
    );
    push_line(output, "    /// native trace file cannot be opened.");
    push_line(output, "    pub fn open_trace(");
    push_line(output, "        &mut self,");
    push_line(output, "        path: &std::path::Path,");
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");
    push_line(output, "        if self.evaluated.is_set() {");
    push_line(
        output,
        &format!(
            "            return Err({}::TraceAfterEvaluation);",
            names.rust_error_type
        ),
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        if self.trace_configured.is_set() {");
    push_line(
        output,
        &format!(
            "            return Err({}::TraceAlreadyConfigured);",
            names.rust_error_type
        ),
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        let path = path.to_str().ok_or(");
    push_line(
        output,
        &format!("            {}::TracePathNotUtf8,", names.rust_error_type),
    );
    push_line(output, "        )?;");
    push_line(output, "");
    push_line(output, "        let opened =");
    push_line(output, "            self.inner_mut()?.open_trace(path);");
    push_line(output, "");
    push_line(
        output,
        "        self.trace_configured = TraceFlag::new(true);",
    );
    push_line(output, "");
    push_line(output, "        if !opened {");
    push_line(
        output,
        &format!(
            "            return Err({}::TraceOpenFailed);",
            names.rust_error_type
        ),
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(
        output,
        "        self.trace_open = TraceFlag::new(self.inner_ref()?.trace_is_open());",
    );
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
}

/// Renders the safe trace closing and query methods.
fn render_close_trace_methods(output: &mut String) {
    push_line(output, "");
    push_line(
        output,
        "    /// Flushes and closes the active waveform trace.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// Repeated calls are safe.");
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the native adapter is unexpectedly unavailable.",
    );
    push_line(output, "    pub fn close_trace(&mut self) -> Result<()> {");
    push_line(output, "        if self.finished {");
    push_line(
        output,
        "            self.trace_open = TraceFlag::new(false);",
    );
    push_line(output, "            return Ok(());");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        self.inner_mut()?.close_trace();");
    push_line(
        output,
        "        self.trace_open = TraceFlag::new(self.inner_ref()?.trace_is_open());",
    );
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");

    push_line(output, "");
    push_line(
        output,
        "    /// Returns whether the waveform trace is currently open.",
    );
    push_line(output, "    #[must_use]");
    push_line(output, "    pub const fn trace_is_open(&self) -> bool {");
    push_line(output, "        self.trace_open.is_set()");
    push_line(output, "    }");
}

/// Renders timing operations.
fn render_timing(output: &mut String, names: &DutNames) {
    push_line(output, "    /// Returns the current simulation time.");
    push_line(output, "    #[must_use]");
    push_line(output, "    const fn simulation_time_value(");
    push_line(output, "        &self,");
    push_line(output, "    ) -> ::vvm::SimulationTime {");
    push_line(output, "        self.time");
    push_line(output, "    }");

    push_line(
        output,
        "    /// Advances the simulation time without evaluating the DUT.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT is finished, the adapter is unavailable, or",
    );
    push_line(output, "    /// the resulting time would overflow.");
    push_line(output, "    fn advance_time_inner(");
    push_line(output, "        &mut self,");
    push_line(output, "        delta: ::vvm::TimeStep,");
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        self.ensure_running()?;");

    push_line(output, "        let Some(next_time) =");
    push_line(output, "            self.time.checked_add(delta)");
    push_line(output, "        else {");
    push_line(
        output,
        &format!(
            "            return Err({}::TimeOverflow);",
            names.rust_error_type
        ),
    );
    push_line(output, "        };");

    push_line(output, "        let advanced = self");
    push_line(output, "            .inner_mut()?");
    push_line(output, "            .advance_time(delta.ticks());");

    push_line(output, "        if !advanced {");
    push_line(
        output,
        &format!(
            "            return Err({}::TimeOverflow);",
            names.rust_error_type
        ),
    );
    push_line(output, "        }");

    push_line(output, "        self.time = next_time;");

    push_line(output, "        Ok(())");
    push_line(output, "    }");

    let _ = names;
}

/// Renders one typed input setter.
fn render_input(output: &mut String, port: &Port, port_names: &PortNames, names: &DutNames) {
    if let Some(array_type) = PackedArrayType::from_port(port) {
        render_packed_array_input(output, port, port_names, array_type, names);

        return;
    }

    match PortType::from_port(port) {
        PortType::Scalar(signal_type) => {
            render_scalar_input(output, port, &port_names.method, signal_type);
        }
        PortType::Wide(wide_type) => {
            render_wide_input(output, port, &port_names.method, wide_type, names);
        }
    }
}

/// Renders one scalar typed input setter.
fn render_scalar_input(output: &mut String, port: &Port, method: &str, signal_type: SignalType) {
    let rust_type = signal_type.rust_type();

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Drives the `{}` DUT input.", port.name),
    );
    push_line(output, "    ///");
    push_line(
        output,
        "    /// The value may be supplied by value or by reference.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT has already been finished.",
    );

    push_line(output, "    #[allow(clippy::needless_pass_by_value)]");

    push_line(output, &format!("    pub fn {method}("));
    push_line(output, "        &mut self,");
    push_line(
        output,
        &format!("        value: impl ::core::borrow::Borrow<{rust_type}>,"),
    );
    push_line(output, "    ) -> Result<()> {");

    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");

    push_line(output, &format!("        let value: {rust_type} ="));
    push_line(
        output,
        "            *::core::borrow::Borrow::borrow(&value);",
    );
    push_line(output, "");

    push_line(
        output,
        &format!("        self.inner_mut()?.{method}(value);"),
    );

    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
}

/// Renders one typed output getter.
fn render_output(output: &mut String, port: &Port, port_names: &PortNames, names: &DutNames) {
    if let Some(array_type) = PackedArrayType::from_port(port) {
        render_packed_array_output(output, port, port_names, array_type, names);

        return;
    }

    match PortType::from_port(port) {
        PortType::Scalar(signal_type) => {
            render_scalar_output(output, port, &port_names.method, signal_type);
        }
        PortType::Wide(wide_type) => {
            render_wide_output(output, port, &port_names.method, wide_type, names);
        }
    }
}

/// Renders one packed-array typed input setter.
fn render_packed_array_input(
    output: &mut String,
    port: &Port,
    port_names: &PortNames,
    array_type: PackedArrayType<'_>,
    names: &DutNames,
) {
    let Some(rust_type) = port_names.rust_type.as_deref() else {
        return;
    };

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Drives the `{}` DUT input.", port.name),
    );
    push_line(output, "    ///");
    push_line(
        output,
        "    /// The value may be supplied by value or by reference.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT has already been finished, a generated",
    );
    push_line(
        output,
        "    /// packed-array layout conversion fails, or a native wide transfer is rejected.",
    );
    push_line(output, "    #[allow(clippy::needless_pass_by_value)]");
    push_line(output, &format!("    pub fn {}(", port_names.method));
    push_line(output, "        &mut self,");
    push_line(
        output,
        &format!("        value: impl ::core::borrow::Borrow<{rust_type}>,"),
    );
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");
    push_line(output, "        let value =");
    push_line(
        output,
        "            ::core::borrow::Borrow::borrow(&value);",
    );
    push_line(output, "");

    match array_type.storage_port_type() {
        PortType::Scalar(signal_type) => {
            render_packed_array_scalar_input_body(
                output,
                port_names,
                array_type,
                signal_type,
                names,
            );
        }
        PortType::Wide(_) => {
            push_line(output, "        let transferred =");
            push_line(
                output,
                &format!("            self.inner_mut()?.{}(", port_names.method),
            );
            push_line(output, "                value.words_le(),");
            push_line(output, "            );");
            push_line(output, "");
            push_line(output, "        if !transferred {");
            push_line(output, "            return Err(");
            push_line(
                output,
                &format!(
                    "                {}::WidePortTransferFailed,",
                    names.rust_error_type
                ),
            );
            push_line(output, "            );");
            push_line(output, "        }");
        }
    }

    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
}

/// Renders the body of one flattened-scalar packed-array input setter.
#[allow(clippy::too_many_lines)]
fn render_packed_array_scalar_input_body(
    output: &mut String,
    port_names: &PortNames,
    array_type: PackedArrayType<'_>,
    signal_type: SignalType,
    names: &DutNames,
) {
    if signal_type == SignalType::Bool {
        push_line(output, "        let raw = ::vvm::extract_unsigned(");
        push_line(output, "            value.words_le(),");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "            0,");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "        )");
        push_line(
            output,
            &format!(
                "        .map_err(|_error| {}::PackedLayoutFailed)?;",
                names.rust_error_type
            ),
        );
        push_line(output, "");
        push_line(output, "        let raw = raw != 0;");
    } else if array_type.storage_signed() {
        push_line(output, "        let raw = ::vvm::extract_signed(");
        push_line(output, "            value.words_le(),");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "            0,");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "        )");
        push_line(
            output,
            &format!(
                "        .map_err(|_error| {}::PackedLayoutFailed)?;",
                names.rust_error_type
            ),
        );
        push_line(output, "");
        push_line(
            output,
            &format!(
                "        let raw: {} = {}::try_from(raw)",
                signal_type.rust_type(),
                signal_type.rust_type()
            ),
        );
        push_line(
            output,
            &format!(
                "            .map_err(|_error| {}::PackedLayoutFailed)?;",
                names.rust_error_type
            ),
        );
    } else {
        push_line(output, "        let raw = ::vvm::extract_unsigned(");
        push_line(output, "            value.words_le(),");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "            0,");
        push_line(
            output,
            &format!(
                "            {0}::WIDTH,",
                port_names.rust_type.as_deref().unwrap_or("")
            ),
        );
        push_line(output, "        )");
        push_line(
            output,
            &format!(
                "        .map_err(|_error| {}::PackedLayoutFailed)?;",
                names.rust_error_type
            ),
        );
        push_line(output, "");
        push_line(
            output,
            &format!(
                "        let raw: {} = {}::try_from(raw)",
                signal_type.rust_type(),
                signal_type.rust_type()
            ),
        );
        push_line(
            output,
            &format!(
                "            .map_err(|_error| {}::PackedLayoutFailed)?;",
                names.rust_error_type
            ),
        );
    }

    push_line(output, "");
    push_line(
        output,
        &format!("        self.inner_mut()?.{}(raw);", port_names.method),
    );
}

/// Renders one packed-array typed output getter.
fn render_packed_array_output(
    output: &mut String,
    port: &Port,
    port_names: &PortNames,
    array_type: PackedArrayType<'_>,
    names: &DutNames,
) {
    let Some(rust_type) = port_names.rust_type.as_deref() else {
        return;
    };

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
        "    /// Returns an error if the DUT has already been finished, a generated",
    );
    push_line(
        output,
        "    /// packed-array layout conversion fails, or a native wide transfer is rejected.",
    );
    push_line(
        output,
        &format!(
            "    pub fn {}(&self) -> Result<{rust_type}> {{",
            port_names.method
        ),
    );
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");

    match array_type.storage_port_type() {
        PortType::Scalar(signal_type) => {
            render_packed_array_scalar_output_body(output, port_names, array_type, signal_type);
        }
        PortType::Wide(_) => {
            let storage_constructor_type = array_type.storage_constructor_type();
            push_line(output, "        let mut words =");
            push_line(
                output,
                &format!("            vec![0_u32; {rust_type}::WORDS];"),
            );
            push_line(output, "");
            push_line(output, "        let transferred =");
            push_line(
                output,
                &format!(
                    "            self.inner_ref()?.{}(&mut words);",
                    port_names.method
                ),
            );
            push_line(output, "");
            push_line(output, "        if !transferred {");
            push_line(output, "            return Err(");
            push_line(
                output,
                &format!(
                    "                {}::WidePortTransferFailed,",
                    names.rust_error_type
                ),
            );
            push_line(output, "            );");
            push_line(output, "        }");
            push_line(output, "");
            push_line(
                output,
                &format!("        let bits = {storage_constructor_type}::from_words_le(words)"),
            );
            push_line(
                output,
                &format!(
                    "            .map_err(|_error| {}::PackedLayoutFailed)?;",
                    names.rust_error_type
                ),
            );
            push_line(output, "");
            push_line(output, &format!("        Ok({rust_type}::from_bits(bits))"));
        }
    }

    push_line(output, "    }");
}

/// Renders the body of one flattened-scalar packed-array output getter.
fn render_packed_array_scalar_output_body(
    output: &mut String,
    port_names: &PortNames,
    array_type: PackedArrayType<'_>,
    signal_type: SignalType,
) {
    let Some(rust_type) = port_names.rust_type.as_deref() else {
        return;
    };

    let storage_constructor_type = array_type.storage_constructor_type();

    push_line(
        output,
        &format!(
            "        let raw = self.inner_ref()?.{}();",
            port_names.method
        ),
    );
    push_line(output, "");

    push_line(
        output,
        &format!("        let bits = {storage_constructor_type}::from("),
    );

    if signal_type == SignalType::Bool {
        push_line(output, "            u64::from(u8::from(raw)),");
    } else if array_type.storage_signed() {
        push_line(output, "            i64::from(raw),");
    } else {
        push_line(output, "            u64::from(raw),");
    }
    push_line(output, "        );");

    push_line(output, "");
    push_line(output, &format!("        Ok({rust_type}::from_bits(bits))"));
}

/// Renders one scalar typed output getter.
fn render_scalar_output(output: &mut String, port: &Port, method: &str, signal_type: SignalType) {
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

/// Renders one wide typed input setter.
fn render_wide_input(
    output: &mut String,
    port: &Port,
    method: &str,
    wide_type: WideType,
    names: &DutNames,
) {
    let rust_type = wide_type.rust_value_type();

    push_line(output, "");
    push_line(
        output,
        &format!("    /// Drives the `{}` DUT input.", port.name),
    );
    push_line(output, "    ///");
    push_line(
        output,
        "    /// The value may be supplied by value or by reference.",
    );
    push_line(output, "    ///");
    push_line(output, "    /// # Errors");
    push_line(output, "    ///");
    push_line(
        output,
        "    /// Returns an error if the DUT has already been finished or the native",
    );
    push_line(output, "    /// wide-port transfer is rejected.");
    push_line(output, "    #[allow(clippy::needless_pass_by_value)]");
    push_line(output, &format!("    pub fn {method}("));
    push_line(output, "        &mut self,");
    push_line(
        output,
        &format!("        value: impl ::core::borrow::Borrow<{rust_type}>,"),
    );
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");
    push_line(output, "        let value =");
    push_line(
        output,
        "            ::core::borrow::Borrow::borrow(&value);",
    );
    push_line(output, "");
    push_line(output, "        let transferred =");
    push_line(output, &format!("            self.inner_mut()?.{method}("));
    push_line(output, "                value.words_le(),");
    push_line(output, "            );");
    push_line(output, "");
    push_line(output, "        if !transferred {");
    push_line(output, "            return Err(");
    push_line(
        output,
        &format!(
            "                {}::WidePortTransferFailed,",
            names.rust_error_type
        ),
    );
    push_line(output, "            );");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        Ok(())");
    push_line(output, "    }");
}

/// Renders one wide typed output getter.
fn render_wide_output(
    output: &mut String,
    port: &Port,
    method: &str,
    wide_type: WideType,
    names: &DutNames,
) {
    let rust_type = wide_type.rust_value_type();
    let constructor_type = wide_type.rust_constructor_type();

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
        "    /// Returns an error if the DUT has already been finished or the native",
    );
    push_line(output, "    /// wide-port transfer is rejected.");
    push_line(
        output,
        &format!("    pub fn {method}(&self) -> Result<{rust_type}> {{"),
    );
    push_line(output, "        self.ensure_running()?;");
    push_line(output, "");
    push_line(output, "        let mut words =");
    push_line(
        output,
        &format!("            vec![0_u32; {constructor_type}::WORDS];"),
    );
    push_line(output, "");
    push_line(output, "        let transferred =");
    push_line(
        output,
        &format!("            self.inner_ref()?.{method}(&mut words);"),
    );
    push_line(output, "");
    push_line(output, "        if !transferred {");
    push_line(output, "            return Err(");
    push_line(
        output,
        &format!(
            "                {}::WidePortTransferFailed,",
            names.rust_error_type
        ),
    );
    push_line(output, "            );");
    push_line(output, "        }");
    push_line(output, "");
    push_line(
        output,
        &format!("        {constructor_type}::from_words_le(words)"),
    );
    push_line(output, "            .map_err(|_error| {");
    push_line(
        output,
        &format!(
            "                {}::WidePortTransferFailed",
            names.rust_error_type
        ),
    );
    push_line(output, "            })");
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
fn render_debug(output: &mut String, names: &DutNames, traced: bool) {
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
    push_line(output, "            .field(\"time\", &self.time)");
    if traced {
        push_line(output, "            .field(\"evaluated\", &self.evaluated)");
        push_line(
            output,
            "            .field(\"trace_configured\", &self.trace_configured)",
        );
        push_line(
            output,
            "            .field(\"trace_open\", &self.trace_open)",
        );
    }
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

    push_line(output, "    fn simulation_time(");
    push_line(output, "        &self,");
    push_line(output, "    ) -> ::vvm::SimulationTime {");
    push_line(output, "        Self::simulation_time_value(self)");
    push_line(output, "    }");

    push_line(output, "    fn advance_time(");
    push_line(output, "        &mut self,");
    push_line(output, "        delta: ::vvm::TimeStep,");
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        Self::advance_time_inner(self, delta)");
    push_line(output, "    }");

    push_line(output, "}");
}

/// Renders the optional VVM traceable DUT trait implementation.
fn render_traceable_trait(output: &mut String, names: &DutNames) {
    push_line(output, "#[allow(clippy::same_name_method)]");
    push_line(
        output,
        &format!("impl ::vvm::TraceableDut for {} {{", names.cpp_type),
    );
    push_line(output, "    fn open_trace(");
    push_line(output, "        &mut self,");
    push_line(output, "        path: &std::path::Path,");
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        Self::open_trace(self, path)");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    fn close_trace(");
    push_line(output, "        &mut self,");
    push_line(output, "    ) -> Result<()> {");
    push_line(output, "        Self::close_trace(self)");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    fn trace_is_open(&self) -> bool {");
    push_line(output, "        Self::trace_is_open(self)");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Renders safe drop-time finalisation.
fn render_drop(output: &mut String, names: &DutNames, traced: bool) {
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
    if traced {
        push_line(output, "        self.trace_open = TraceFlag::new(false);");
    }
    push_line(output, "        self.finished = true;");
    push_line(output, "    }");
    push_line(output, "}");
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
