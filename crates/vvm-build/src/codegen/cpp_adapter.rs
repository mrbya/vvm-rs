use super::names::DutNames;
use super::types::{
    PortType, SignalType, UnpackedArrayElementType, UnpackedArrayType, WideType,
    contains_slice_ports,
};
use crate::TraceOptions;
use crate::codegen::GENERATED_NOTICE;
use crate::metadata::{DutMetadata, Port, PortDirection};
use crate::trace::TraceFormat;

/// Complete generated C++ adapter text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CppAdapterText {
    /// Generated public header.
    pub(super) header: String,

    /// Generated implementation source.
    pub(super) source: String,
}

/// Renders a complete C++ adapter.
pub(super) fn render(
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
    timing: bool,
) -> CppAdapterText {
    let trace = trace.filter(|options| options.format == TraceFormat::Vcd);
    let requires_rust_cxx = trace.is_some() || contains_slice_ports(metadata);
    let traced = trace.is_some();

    CppAdapterText {
        header: render_header(metadata, names, traced, requires_rust_cxx, timing),
        source: render_source(metadata, names, trace, timing),
    }
}

/// Renders the generated public adapter header.
fn render_header(
    metadata: &DutMetadata,
    names: &DutNames,
    traced: bool,
    include_rust_cxx: bool,
    timing: bool,
) -> String {
    let mut output = String::new();

    render_header_prelude(&mut output, names, traced, include_rust_cxx, timing);
    render_header_port_methods(&mut output, metadata, names);

    push_line(&mut output, "");
    push_line(&mut output, "private:");
    push_line(&mut output, "    class Impl;");
    push_line(&mut output, "    std::unique_ptr<Impl> impl_;");
    push_line(&mut output, "};");
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!(
            "[[nodiscard]] std::unique_ptr<{}> {}() noexcept;",
            names.cpp_type, names.factory
        ),
    );
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("}} // namespace vvm::{}", names.namespace),
    );

    output
}

/// Renders the header prelude and lifecycle declarations.
fn render_header_prelude(
    output: &mut String,
    names: &DutNames,
    traced: bool,
    include_rust_cxx: bool,
    timing: bool,
) {
    push_line(output, GENERATED_NOTICE);
    push_line(output, "");
    push_line(output, "#pragma once");
    push_line(output, "");
    if include_rust_cxx {
        push_line(output, "#include \"rust/cxx.h\"");
        push_line(output, "#include <cstddef>");
    }
    push_line(output, "#include <cstdint>");
    push_line(output, "#include <memory>");
    push_line(output, "");
    push_line(output, &format!("namespace vvm::{} {{", names.namespace));
    push_line(output, "");
    push_line(output, &format!("class {} final {{", names.cpp_type));
    push_line(output, "public:");
    push_line(output, &format!("    {}();", names.cpp_type));
    push_line(output, &format!("    ~{}() noexcept;", names.cpp_type));
    push_line(output, "");
    push_line(
        output,
        &format!(
            "    {}(const {}&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(
        output,
        &format!(
            "    {}& operator=(const {}&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(output, "");
    push_line(
        output,
        &format!("    {}({}&&) = delete;", names.cpp_type, names.cpp_type),
    );
    push_line(
        output,
        &format!(
            "    {}& operator=({}&&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(output, "");
    push_line(output, "    void eval() noexcept;");
    push_line(output, "    void finish() noexcept;");
    push_line(output, "    [[nodiscard]]");
    push_line(
        output,
        "    bool advance_time(std::uint64_t delta) noexcept;",
    );
    if timing {
        push_line(
            output,
            "    [[nodiscard]] bool events_pending() const noexcept;",
        );
        push_line(
            output,
            "    [[nodiscard]] bool next_time_slot(std::uint64_t& time) const noexcept;",
        );
    }
    if traced {
        push_line(output, "    [[nodiscard]]");
        push_line(output, "    bool open_trace(rust::Str path) noexcept;");
        push_line(output, "");
        push_line(output, "    void close_trace() noexcept;");
        push_line(output, "");
        push_line(output, "    [[nodiscard]]");
        push_line(output, "    bool trace_is_open() const noexcept;");
    }
    push_line(output, "");
}

/// Renders per-port method declarations in the public header.
fn render_header_port_methods(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        if let Some(array_type) = UnpackedArrayType::from_port(port) {
            let value_name =
                if matches!(array_type.element_type(), UnpackedArrayElementType::Wide(_)) {
                    "words"
                } else {
                    "values"
                };
            let value_type = array_type.ffi_cpp_element_type();
            let declaration = match port.direction {
                PortDirection::Input => format!(
                    "    [[nodiscard]] bool {}(rust::Slice<const {value_type}> {value_name}) \
                     noexcept;",
                    port_names.method
                ),
                PortDirection::Output => format!(
                    "    [[nodiscard]] bool {}(rust::Slice<{value_type}> {value_name}) const \
                     noexcept;",
                    port_names.method
                ),
                PortDirection::Inout => continue,
            };
            push_line(output, &declaration);
            continue;
        }
        let port_type = PortType::from_port(port);

        match port.direction {
            PortDirection::Input => push_line(
                output,
                &render_header_input_method(port_type, &port_names.method),
            ),
            PortDirection::Output => {
                push_line(
                    output,
                    &render_header_output_method(port_type, &port_names.method),
                );
            }
            PortDirection::Inout => {}
        }
    }
}

/// Renders one generated input declaration in the public header.
fn render_header_input_method(port_type: PortType, method: &str) -> String {
    match port_type {
        PortType::Scalar(signal_type) => {
            format!(
                "    void {method}({} value) noexcept;",
                signal_type.cpp_type()
            )
        }
        PortType::Wide(_) => format!(
            "    [[nodiscard]] bool {method}(rust::Slice<const std::uint32_t> words) noexcept;"
        ),
    }
}

/// Renders one generated output declaration in the public header.
fn render_header_output_method(port_type: PortType, method: &str) -> String {
    match port_type {
        PortType::Scalar(signal_type) => {
            format!(
                "    [[nodiscard]] {} {method}() const noexcept;",
                signal_type.cpp_type()
            )
        }
        PortType::Wide(_) => format!(
            "    [[nodiscard]] bool {method}(rust::Slice<std::uint32_t> words) const noexcept;"
        ),
    }
}

/// Renders the generated adapter implementation.
fn render_source(
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
    timing: bool,
) -> String {
    let mut output = String::new();
    let traced = trace.is_some();
    let trace_depth = trace.map(|options| options.depth);
    let has_wide_ports = contains_slice_ports(metadata);

    render_source_prelude(&mut output, names, traced, has_wide_ports);
    render_impl_class(&mut output, metadata, names, trace);
    render_lifecycle_methods(&mut output, names, trace_depth);
    render_port_methods(&mut output, metadata, names);
    render_time_methods(&mut output, names, timing);
    render_factory_function(&mut output, names);
    render_source_epilogue(&mut output, names);

    output
}

/// Renders the source-file prelude and namespace opening.
fn render_source_prelude(
    output: &mut String,
    names: &DutNames,
    traced: bool,
    has_wide_ports: bool,
) {
    push_line(output, GENERATED_NOTICE);
    push_line(output, "");
    push_line(output, &format!("#include \"{}.hpp\"", names.file_stem));
    push_line(output, "");
    push_line(output, &format!("#include \"{}.h\"", names.model_type));
    push_line(output, "#include \"verilated.h\"");
    if traced {
        push_line(output, "#include \"verilated_vcd_c.h\"");
    }
    push_line(output, "");
    if has_wide_ports {
        push_line(output, "#include <cstddef>");
    }
    push_line(output, "#include <cstdint>");
    if traced {
        push_line(output, "#include <string>");
    }
    push_line(output, "#include <memory>");
    push_line(output, "#include <type_traits>");
    push_line(output, "#include <limits>");
    push_line(output, "");
    push_line(output, &format!("namespace vvm::{} {{", names.namespace));
    push_line(output, "");
}

/// Renders the PIMPL implementation class.
fn render_impl_class(
    output: &mut String,
    metadata: &DutMetadata,
    names: &DutNames,
    trace: Option<TraceOptions>,
) {
    let traced = trace.is_some();

    push_line(output, &format!("class {}::Impl final {{", names.cpp_type));
    push_line(output, "public:");
    push_line(output, "    Impl()");
    if traced {
        push_line(
            output,
            "        : context{std::make_unique<VerilatedContext>()} {",
        );
        push_line(output, "        context->traceEverOn(true);");
        push_line(output, "");
        push_line(
            output,
            &format!(
                "        model = std::make_unique<{}>(context.get());",
                names.model_type
            ),
        );
    } else {
        push_line(
            output,
            "        : context{std::make_unique<VerilatedContext>()},",
        );
        push_line(
            output,
            &format!(
                "          model{{std::make_unique<{}>(context.get())}} {{",
                names.model_type
            ),
        );
    }

    render_input_initializers(output, metadata, names);

    push_line(output, "    }");
    render_signed_output_decoder(output, metadata);
    push_line(output, "");
    if traced {
        push_line(output, "    void dump_trace() noexcept {");
        push_line(
            output,
            "        if (trace == nullptr || !trace->isOpen()) {",
        );
        push_line(output, "            return;");
        push_line(output, "        }");
        push_line(output, "");
        push_line(output, "        const auto now = context->time();");
        push_line(output, "");
        push_line(
            output,
            "        if (has_trace_dump && now <= last_trace_time) {",
        );
        push_line(output, "            return;");
        push_line(output, "        }");
        push_line(output, "");
        push_line(output, "        trace->dump(now);");
        push_line(output, "        last_trace_time = now;");
        push_line(output, "        has_trace_dump = true;");
        push_line(output, "    }");
        push_line(output, "");
        push_line(output, "    void close_trace() noexcept {");
        push_line(
            output,
            "        if (trace == nullptr || !trace->isOpen()) {",
        );
        push_line(output, "            return;");
        push_line(output, "        }");
        push_line(output, "");
        push_line(output, "        trace->flush();");
        push_line(output, "        trace->close();");
        push_line(output, "    }");
        push_line(output, "");
    }
    push_line(output, "    std::unique_ptr<VerilatedContext> context;");
    push_line(
        output,
        &format!("    std::unique_ptr<{}> model;", names.model_type),
    );
    if traced {
        push_line(output, "    std::unique_ptr<VerilatedVcdC> trace;");
        push_line(output, "    std::uint64_t last_trace_time{0};");
        push_line(output, "    bool has_trace_dump{false};");
    }
    push_line(output, "    bool finished{false};");
    push_line(output, "};");
    push_line(output, "");
}

/// Renders input-port zero initialization in the adapter constructor.
fn render_input_initializers(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        if port.direction != PortDirection::Input {
            continue;
        }

        push_line(output, "        {");
        push_line(
            output,
            &format!(
                "            using RawType = std::decay_t<decltype(model->{})>;",
                port_names.model_member
            ),
        );
        push_line(output, "            RawType raw_value{};");
        push_line(
            output,
            &format!(
                "            model->{} = raw_value;",
                port_names.model_member
            ),
        );
        push_line(output, "        }");
    }
}

/// Renders the signed-output decoding helper when required.
fn render_signed_output_decoder(output: &mut String, metadata: &DutMetadata) {
    let required = metadata.ports.iter().any(|port| {
        (port.signed && port.direction == PortDirection::Output)
            || UnpackedArrayType::from_port(port).is_some_and(|array_type| {
                port.direction == PortDirection::Output
                    && array_type.element_signed()
                    && matches!(
                        array_type.element_type(),
                        UnpackedArrayElementType::Scalar(_)
                    )
            })
    });

    if !required {
        return;
    }

    push_line(output, "");
    push_line(
        output,
        "    template <typename Signed, std::uint32_t Width, typename Raw>",
    );
    push_line(output, "    [[nodiscard]]");
    push_line(output, "    static constexpr Signed sign_extend(");
    push_line(output, "        const Raw raw");
    push_line(output, "    ) noexcept {");
    push_line(output, "        static_assert(std::is_integral_v<Signed>);");
    push_line(output, "        static_assert(std::is_signed_v<Signed>);");
    push_line(output, "        static_assert(std::is_integral_v<Raw>);");
    push_line(output, "");
    push_line(
        output,
        "        using Unsigned = std::make_unsigned_t<Signed>;",
    );
    push_line(output, "");
    push_line(output, "        constexpr auto storage_width =");
    push_line(output, "            static_cast<std::uint32_t>(");
    push_line(
        output,
        "                std::numeric_limits<Unsigned>::digits",
    );
    push_line(output, "            );");
    push_line(output, "");
    push_line(output, "        static_assert(Width > 0);");
    push_line(output, "        static_assert(Width <= storage_width);");
    push_line(output, "");
    push_line(output, "        Unsigned mask{");
    push_line(output, "            std::numeric_limits<Unsigned>::max()");
    push_line(output, "        };");
    push_line(output, "");
    push_line(output, "        if constexpr (Width < storage_width) {");
    push_line(output, "            mask = static_cast<Unsigned>(");
    push_line(output, "                static_cast<Unsigned>(");
    push_line(output, "                    Unsigned{1} << Width");
    push_line(output, "                ) - Unsigned{1}");
    push_line(output, "            );");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        const auto value = static_cast<Unsigned>(");
    push_line(output, "            static_cast<Unsigned>(raw) & mask");
    push_line(output, "        );");
    push_line(output, "");
    push_line(output, "        constexpr auto sign_bit =");
    push_line(output, "            static_cast<Unsigned>(");
    push_line(output, "                Unsigned{1} << (Width - 1U)");
    push_line(output, "            );");
    push_line(output, "");
    push_line(output, "        if ((value & sign_bit) == Unsigned{0}) {");
    push_line(output, "            return static_cast<Signed>(value);");
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        const auto magnitude =");
    push_line(output, "            static_cast<Unsigned>(");
    push_line(output, "                static_cast<Unsigned>(");
    push_line(
        output,
        "                    static_cast<Unsigned>(~value) & mask",
    );
    push_line(output, "                ) + Unsigned{1}");
    push_line(output, "            );");
    push_line(output, "");
    push_line(output, "        if (");
    push_line(output, "            magnitude");
    push_line(output, "            > static_cast<Unsigned>(");
    push_line(output, "                std::numeric_limits<Signed>::max()");
    push_line(output, "            )");
    push_line(output, "        ) {");
    push_line(
        output,
        "            return std::numeric_limits<Signed>::min();",
    );
    push_line(output, "        }");
    push_line(output, "");
    push_line(output, "        return static_cast<Signed>(");
    push_line(output, "            -static_cast<Signed>(magnitude)");
    push_line(output, "        );");
    push_line(output, "    }");
}

/// Renders constructor, destructor, and simulation lifecycle methods.
fn render_lifecycle_methods(output: &mut String, names: &DutNames, trace_depth: Option<u32>) {
    let traced = trace_depth.is_some();
    let trace_depth = trace_depth.unwrap_or(0);

    push_line(output, &format!("{}::{}()", names.cpp_type, names.cpp_type));
    push_line(output, "    : impl_{std::make_unique<Impl>()} {}");
    push_line(output, "");
    push_line(
        output,
        &format!("{}::~{}() noexcept {{", names.cpp_type, names.cpp_type),
    );
    push_line(output, "    finish();");
    push_line(output, "}");
    push_line(output, "");
    push_line(
        output,
        &format!("void {}::eval() noexcept {{", names.cpp_type),
    );
    push_line(output, "    if (impl_->finished) {");
    push_line(output, "        return;");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    impl_->model->eval();");
    if traced {
        push_line(output, "    impl_->dump_trace();");
    }
    push_line(output, "}");
    push_line(output, "");
    push_line(
        output,
        &format!("void {}::finish() noexcept {{", names.cpp_type),
    );
    push_line(output, "    if (impl_->finished) {");
    push_line(output, "        return;");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    impl_->model->final();");
    if traced {
        push_line(output, "");
        push_line(output, "    impl_->dump_trace();");
        push_line(output, "    impl_->close_trace();");
    }
    push_line(output, "");
    push_line(output, "    impl_->finished = true;");
    push_line(output, "}");
    if traced {
        push_line(output, "");
        push_line(output, &format!("bool {}::open_trace(", names.cpp_type));
        push_line(output, "    const rust::Str path");
        push_line(output, ") noexcept {");
        push_line(
            output,
            "    if (impl_->finished || impl_->trace != nullptr) {",
        );
        push_line(output, "        return false;");
        push_line(output, "    }");
        push_line(output, "");
        push_line(output, "    try {");
        push_line(
            output,
            "        impl_->trace = std::make_unique<VerilatedVcdC>();",
        );
        push_line(output, "");
        push_line(
            output,
            &format!("        impl_->model->trace(impl_->trace.get(), {trace_depth});"),
        );
        push_line(output, "");
        push_line(output, "        const std::string filename{path};");
        push_line(output, "");
        push_line(output, "        impl_->trace->open(filename.c_str());");
        push_line(output, "");
        push_line(output, "        return impl_->trace->isOpen();");
        push_line(output, "    } catch (...) {");
        push_line(output, "        return false;");
        push_line(output, "    }");
        push_line(output, "}");
        push_line(output, "");
        push_line(
            output,
            &format!("void {}::close_trace() noexcept {{", names.cpp_type),
        );
        push_line(output, "    impl_->close_trace();");
        push_line(output, "}");
        push_line(output, "");
        push_line(
            output,
            &format!("bool {}::trace_is_open() const noexcept {{", names.cpp_type),
        );
        push_line(
            output,
            "    return impl_->trace != nullptr && impl_->trace->isOpen();",
        );
        push_line(output, "}");
    }
}

/// Renders generated per-port setters and getters.
fn render_port_methods(output: &mut String, metadata: &DutMetadata, names: &DutNames) {
    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        match port.direction {
            PortDirection::Input => {
                render_setter(
                    output,
                    port,
                    &port_names.method,
                    &port_names.model_member,
                    &names.cpp_type,
                );
            }
            PortDirection::Output => {
                render_getter(
                    output,
                    port,
                    &port_names.method,
                    &port_names.model_member,
                    &names.cpp_type,
                );
            }
            PortDirection::Inout => {}
        }
    }
}

/// Renders the adapter timing methods.
fn render_time_methods(output: &mut String, names: &DutNames, timing: bool) {
    push_line(output, "");
    push_line(output, &format!("bool {}::advance_time(", names.cpp_type));
    push_line(output, "    const std::uint64_t delta");
    push_line(output, ") noexcept {");
    push_line(output, "    const auto current = impl_->context->time();");
    push_line(output, "");
    push_line(
        output,
        "    if (delta > std::numeric_limits<std::uint64_t>::max() - current) {",
    );
    push_line(output, "        return false;");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    impl_->context->timeInc(delta);");
    push_line(output, "");
    push_line(output, "    return true;");
    push_line(output, "}");
    if timing {
        push_line(output, "");
        push_line(
            output,
            &format!(
                "bool {}::events_pending() const noexcept {{",
                names.cpp_type
            ),
        );
        push_line(output, "    if (impl_->finished) {");
        push_line(output, "        return false;");
        push_line(output, "    }");
        push_line(output, "");
        push_line(output, "    return impl_->model->eventsPending();");
        push_line(output, "}");
        push_line(output, "");
        push_line(output, &format!("bool {}::next_time_slot(", names.cpp_type));
        push_line(output, "    std::uint64_t& time");
        push_line(output, ") const noexcept {");
        push_line(
            output,
            "    if (impl_->finished || !impl_->model->eventsPending()) {",
        );
        push_line(output, "        return false;");
        push_line(output, "    }");
        push_line(output, "");
        push_line(output, "    time = impl_->model->nextTimeSlot();");
        push_line(output, "");
        push_line(output, "    return true;");
        push_line(output, "}");
    }
}

/// Renders the adapter factory function.
fn render_factory_function(output: &mut String, names: &DutNames) {
    push_line(output, "");
    push_line(
        output,
        &format!(
            "std::unique_ptr<{}> {}() noexcept {{",
            names.cpp_type, names.factory
        ),
    );
    push_line(output, "    try {");
    push_line(
        output,
        &format!("        return std::make_unique<{}>();", names.cpp_type),
    );
    push_line(output, "    } catch (...) {");
    push_line(output, "        return nullptr;");
    push_line(output, "    }");
    push_line(output, "}");
    push_line(output, "");
}

/// Renders the namespace closing line.
fn render_source_epilogue(output: &mut String, names: &DutNames) {
    push_line(output, &format!("}} // namespace vvm::{}", names.namespace));
}

/// Renders one generated input setter.
fn render_setter(
    output: &mut String,
    port: &Port,
    method: &str,
    model_member: &str,
    cpp_type: &str,
) {
    if let Some(array_type) = UnpackedArrayType::from_port(port) {
        render_unpacked_array_setter(output, method, model_member, cpp_type, array_type);
        return;
    }
    match PortType::from_port(port) {
        PortType::Scalar(signal_type) => {
            render_scalar_setter(output, port, method, model_member, cpp_type, signal_type);
        }
        PortType::Wide(wide_type) => {
            render_wide_setter(output, method, model_member, cpp_type, wide_type);
        }
    }
}

/// Renders one scalar input setter.
fn render_scalar_setter(
    output: &mut String,
    port: &Port,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    signal_type: SignalType,
) {
    push_line(output, "");
    push_line(
        output,
        &format!(
            "void {cpp_type}::{method}(const {} value) noexcept {{",
            signal_type.cpp_type()
        ),
    );
    push_line(
        output,
        &format!("    using RawType = std::decay_t<decltype(impl_->model->{model_member})>;"),
    );

    if port.signed {
        push_line(
            output,
            &format!(
                "    using UnsignedType = {};",
                signal_type.unsigned_cpp_type()
            ),
        );
        push_line(output, "");
        push_line(
            output,
            "    const auto bits = static_cast<UnsignedType>(value);",
        );

        let value_expression = signal_type
            .mask_literal(port.width)
            .map_or_else(|| "bits".to_owned(), |mask| format!("bits & {mask}"));

        push_line(
            output,
            &format!("    RawType raw_value{{static_cast<RawType>({value_expression})}};"),
        );
    } else {
        let value_expression = signal_type
            .mask_literal(port.width)
            .map_or_else(|| "value".to_owned(), |mask| format!("value & {mask}"));

        push_line(output, "");
        push_line(
            output,
            &format!("    RawType raw_value{{static_cast<RawType>({value_expression})}};"),
        );
    }

    push_line(
        output,
        &format!("    impl_->model->{model_member} = raw_value;"),
    );
    push_line(output, "}");
}

/// Renders one wide input setter.
fn render_wide_setter(
    output: &mut String,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    wide_type: WideType,
) {
    let expected_words = wide_type.word_count();
    let final_index = wide_type.word_count().saturating_sub(1);

    push_line(output, "");
    push_line(
        output,
        &format!("bool {cpp_type}::{method}(rust::Slice<const std::uint32_t> words) noexcept {{"),
    );
    push_line(
        output,
        &format!("    constexpr std::size_t expected_words{{{expected_words}}};"),
    );
    push_line(output, "");
    push_line(output, "    if (words.size() != expected_words) {");
    push_line(output, "        return false;");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    using RawType =");
    push_line(output, "        std::decay_t<decltype(");
    push_line(output, &format!("            impl_->model->{model_member}"));
    push_line(output, "        )>;");
    push_line(output, "");
    push_line(
        output,
        "    static_assert(RawType::Words == expected_words);",
    );
    push_line(output, "");
    push_line(output, "    RawType raw_value{};");
    push_line(output, "");
    push_line(output, "    for (");
    push_line(output, "        std::size_t index = 0;");
    push_line(output, "        index < expected_words;");
    push_line(output, "        ++index");
    push_line(output, "    ) {");
    push_line(output, "        raw_value[index] = words[index];");
    push_line(output, "    }");

    if wide_type.requires_final_word_mask() {
        push_line(output, "");
        push_line(
            output,
            &format!(
                "    raw_value[{final_index}] &= {};",
                wide_type.final_word_mask_literal()
            ),
        );
    }

    push_line(output, "");
    push_line(
        output,
        &format!("    impl_->model->{model_member} = raw_value;"),
    );
    push_line(output, "");
    push_line(output, "    return true;");
    push_line(output, "}");
}

/// Renders one generated output getter.
fn render_getter(
    output: &mut String,
    port: &Port,
    method: &str,
    model_member: &str,
    cpp_type: &str,
) {
    if let Some(array_type) = UnpackedArrayType::from_port(port) {
        render_unpacked_array_getter(output, method, model_member, cpp_type, array_type);
        return;
    }
    match PortType::from_port(port) {
        PortType::Scalar(signal_type) => {
            render_scalar_getter(output, port, method, model_member, cpp_type, signal_type);
        }
        PortType::Wide(wide_type) => {
            render_wide_getter(output, method, model_member, cpp_type, wide_type);
        }
    }
}

/// Renders one scalar output getter.
fn render_scalar_getter(
    output: &mut String,
    port: &Port,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    signal_type: SignalType,
) {
    push_line(output, "");
    push_line(
        output,
        &format!(
            "{} {cpp_type}::{method}() const noexcept {{",
            signal_type.cpp_type()
        ),
    );

    if signal_type == SignalType::Bool {
        push_line(
            output,
            &format!("    return impl_->model->{model_member} != 0;"),
        );
    } else if port.signed {
        push_line(
            output,
            &format!(
                "    return Impl::sign_extend<{}, {}>(",
                signal_type.cpp_type(),
                port.width.get(),
            ),
        );
        push_line(output, &format!("        impl_->model->{model_member}"));
        push_line(output, "    );");
    } else if let Some(mask) = signal_type.mask_literal(port.width) {
        push_line(
            output,
            &format!(
                "    return static_cast<{}>(impl_->model->{model_member} & {mask});",
                signal_type.cpp_type()
            ),
        );
    } else {
        push_line(
            output,
            &format!(
                "    return static_cast<{}>(impl_->model->{model_member});",
                signal_type.cpp_type()
            ),
        );
    }

    push_line(output, "}");
}

/// Renders one wide output getter.
fn render_wide_getter(
    output: &mut String,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    wide_type: WideType,
) {
    let expected_words = wide_type.word_count();
    let final_index = wide_type.word_count().saturating_sub(1);

    push_line(output, "");
    push_line(
        output,
        &format!("bool {cpp_type}::{method}(rust::Slice<std::uint32_t> words) const noexcept {{"),
    );
    push_line(
        output,
        &format!("    constexpr std::size_t expected_words{{{expected_words}}};"),
    );
    push_line(output, "");
    push_line(output, "    if (words.size() != expected_words) {");
    push_line(output, "        return false;");
    push_line(output, "    }");
    push_line(output, "");
    push_line(output, "    using RawType =");
    push_line(output, "        std::decay_t<decltype(");
    push_line(output, &format!("            impl_->model->{model_member}"));
    push_line(output, "        )>;");
    push_line(output, "");
    push_line(
        output,
        "    static_assert(RawType::Words == expected_words);",
    );
    push_line(output, "");
    push_line(output, "    const auto& raw_value =");
    push_line(output, &format!("        impl_->model->{model_member};"));
    push_line(output, "");
    push_line(output, "    for (");
    push_line(output, "        std::size_t index = 0;");
    push_line(output, "        index < expected_words;");
    push_line(output, "        ++index");
    push_line(output, "    ) {");
    push_line(output, "        words[index] = raw_value[index];");
    push_line(output, "    }");

    if wide_type.requires_final_word_mask() {
        push_line(output, "");
        push_line(
            output,
            &format!(
                "    words[{final_index}] &= {};",
                wide_type.final_word_mask_literal()
            ),
        );
    }

    push_line(output, "");
    push_line(output, "    return true;");
    push_line(output, "}");
}

/// Renders one unpacked-array input transfer.
fn render_unpacked_array_setter(
    output: &mut String,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    array: UnpackedArrayType<'_>,
) {
    let Some(transfer_length) = array.transfer_length() else {
        return;
    };
    let value_name = unpacked_array_value_name(array);
    render_unpacked_array_setter_prelude(
        output,
        model_member,
        cpp_type,
        method,
        array,
        transfer_length,
        value_name,
    );
    match array.element_type() {
        UnpackedArrayElementType::Scalar(signal) => {
            render_unpacked_scalar_setter(output, array, signal);
        }
        UnpackedArrayElementType::Wide(wide) => render_unpacked_wide_setter(output, array, wide),
    }
    push_line(
        output,
        &format!("    impl_->model->{model_member} = raw_value;"),
    );
    push_line(output, "    return true;");
    push_line(output, "}");
}

/// Returns the generated C++ transfer parameter name for an unpacked array.
const fn unpacked_array_value_name(array: UnpackedArrayType<'_>) -> &'static str {
    if matches!(array.element_type(), UnpackedArrayElementType::Wide(_)) {
        "words"
    } else {
        "values"
    }
}

/// Renders common unpacked-array input validation and storage setup.
fn render_unpacked_array_setter_prelude(
    output: &mut String,
    model_member: &str,
    cpp_type: &str,
    method: &str,
    array: UnpackedArrayType<'_>,
    transfer_length: u32,
    value_name: &str,
) {
    let value_type = array.ffi_cpp_element_type();
    push_line(output, "");
    push_line(
        output,
        &format!(
            "bool {cpp_type}::{method}(rust::Slice<const {value_type}> {value_name}) noexcept {{"
        ),
    );
    push_line(
        output,
        &format!(
            "    constexpr std::size_t expected_elements{{{}}};",
            array.length()
        ),
    );
    push_line(
        output,
        &format!("    constexpr std::size_t expected_transfer{{{transfer_length}}};"),
    );
    push_line(
        output,
        &format!("    if ({value_name}.size() != expected_transfer) {{"),
    );
    push_line(output, "        return false;");
    push_line(output, "    }");
    push_line(
        output,
        &format!("    using RawType = std::decay_t<decltype(impl_->model->{model_member})>;"),
    );
    push_line(output, "    RawType raw_value{};");
    push_line(output, "    if (raw_value.size() != expected_elements) {");
    push_line(output, "        return false;");
    push_line(output, "    }");
}

/// Renders scalar-element copies for an unpacked-array input.
fn render_unpacked_scalar_setter(
    output: &mut String,
    array: UnpackedArrayType<'_>,
    signal: SignalType,
) {
    push_line(
        output,
        "    using RawElement = std::decay_t<decltype(raw_value[0])>;",
    );
    push_line(
        output,
        "    for (std::size_t ordinal = 0; ordinal < expected_elements; ++ordinal) {",
    );
    render_unpacked_native_ordinal(output, array, "ordinal");
    let source = render_unpacked_scalar_setter_source(output, array, signal);
    push_line(
        output,
        &format!("        raw_value[native_ordinal] = static_cast<RawElement>({source});"),
    );
    push_line(output, "    }");
}

/// Renders the C++ source expression for one scalar unpacked-array input element.
fn render_unpacked_scalar_setter_source(
    output: &mut String,
    array: UnpackedArrayType<'_>,
    signal: SignalType,
) -> String {
    if array.element_is_bool() {
        return "values[ordinal] != 0U".to_owned();
    }
    if array.element_signed() {
        let mask = signal.mask_literal(array.element_bit_width());
        push_line(
            output,
            &format!(
                "        using UnsignedType = {};",
                signal.unsigned_cpp_type()
            ),
        );
        push_line(
            output,
            "        const auto bits = static_cast<UnsignedType>(values[ordinal]);",
        );
        return mask.map_or_else(|| "bits".to_owned(), |mask| format!("bits & {mask}"));
    }
    signal.mask_literal(array.element_bit_width()).map_or_else(
        || "values[ordinal]".to_owned(),
        |mask| format!("values[ordinal] & {mask}"),
    )
}

/// Renders wide-element copies for an unpacked-array input.
fn render_unpacked_wide_setter(output: &mut String, array: UnpackedArrayType<'_>, wide: WideType) {
    push_line(
        output,
        &format!(
            "    constexpr std::size_t element_words{{{}}};",
            wide.word_count()
        ),
    );
    push_line(
        output,
        "    for (std::size_t element = 0; element < expected_elements; ++element) {",
    );
    render_unpacked_native_ordinal(output, array, "element");
    push_line(
        output,
        "        for (std::size_t word = 0; word < element_words; ++word) {",
    );
    push_line(
        output,
        "            raw_value[native_element][word] = words[element * element_words + word];",
    );
    push_line(output, "        }");
    if wide.requires_final_word_mask() {
        push_line(
            output,
            &format!(
                "        raw_value[native_element][{}] &= {};",
                wide.word_count().saturating_sub(1),
                wide.final_word_mask_literal()
            ),
        );
    }
    push_line(output, "    }");
}

/// Renders an HDL-to-native unpacked-array index conversion.
fn render_unpacked_native_ordinal(
    output: &mut String,
    array: UnpackedArrayType<'_>,
    ordinal: &str,
) {
    let name = if ordinal == "ordinal" {
        "native_ordinal"
    } else {
        "native_element"
    };
    if array.left() > array.right() {
        push_line(
            output,
            &format!("        const auto {name} = expected_elements - 1U - {ordinal};"),
        );
    } else {
        push_line(output, &format!("        const auto {name} = {ordinal};"));
    }
}

/// Renders one unpacked-array output transfer.
fn render_unpacked_array_getter(
    output: &mut String,
    method: &str,
    model_member: &str,
    cpp_type: &str,
    array: UnpackedArrayType<'_>,
) {
    let Some(transfer_length) = array.transfer_length() else {
        return;
    };
    let value_name = unpacked_array_value_name(array);
    render_unpacked_array_getter_prelude(
        output,
        model_member,
        cpp_type,
        method,
        array,
        transfer_length,
        value_name,
    );
    match array.element_type() {
        UnpackedArrayElementType::Scalar(signal) => {
            render_unpacked_scalar_getter(output, array, signal);
        }
        UnpackedArrayElementType::Wide(wide) => render_unpacked_wide_getter(output, array, wide),
    }
    push_line(output, "    return true;");
    push_line(output, "}");
}

/// Renders common unpacked-array output validation and storage access.
fn render_unpacked_array_getter_prelude(
    output: &mut String,
    model_member: &str,
    cpp_type: &str,
    method: &str,
    array: UnpackedArrayType<'_>,
    transfer_length: u32,
    value_name: &str,
) {
    let value_type = array.ffi_cpp_element_type();
    push_line(output, "");
    push_line(
        output,
        &format!(
            "bool {cpp_type}::{method}(rust::Slice<{value_type}> {value_name}) const noexcept {{"
        ),
    );
    push_line(
        output,
        &format!(
            "    constexpr std::size_t expected_elements{{{}}};",
            array.length()
        ),
    );
    push_line(
        output,
        &format!("    constexpr std::size_t expected_transfer{{{transfer_length}}};"),
    );
    push_line(
        output,
        &format!("    if ({value_name}.size() != expected_transfer) {{"),
    );
    push_line(output, "        return false;");
    push_line(output, "    }");
    push_line(
        output,
        &format!("    const auto& raw_value = impl_->model->{model_member};"),
    );
    push_line(output, "    if (raw_value.size() != expected_elements) {");
    push_line(output, "        return false;");
    push_line(output, "    }");
}

/// Renders scalar-element copies for an unpacked-array output.
fn render_unpacked_scalar_getter(
    output: &mut String,
    array: UnpackedArrayType<'_>,
    signal: SignalType,
) {
    push_line(
        output,
        "    for (std::size_t ordinal = 0; ordinal < expected_elements; ++ordinal) {",
    );
    render_unpacked_native_ordinal(output, array, "ordinal");
    if signal == SignalType::Bool {
        push_line(
            output,
            "        values[ordinal] = raw_value[native_ordinal] != 0 ? 1U : 0U;",
        );
    } else if array.element_signed() {
        push_line(
            output,
            &format!(
                "        values[ordinal] = Impl::sign_extend<{}, {}>(raw_value[native_ordinal]);",
                signal.cpp_type(),
                array.element_width()
            ),
        );
    } else {
        let value_type = array.ffi_cpp_element_type();
        push_line(
            output,
            &format!(
                "        values[ordinal] = static_cast<{value_type}>(raw_value[native_ordinal]);"
            ),
        );
    }
    push_line(output, "    }");
}

/// Renders wide-element copies for an unpacked-array output.
fn render_unpacked_wide_getter(output: &mut String, array: UnpackedArrayType<'_>, wide: WideType) {
    push_line(
        output,
        &format!(
            "    constexpr std::size_t element_words{{{}}};",
            wide.word_count()
        ),
    );
    push_line(
        output,
        "    for (std::size_t element = 0; element < expected_elements; ++element) {",
    );
    render_unpacked_native_ordinal(output, array, "element");
    push_line(
        output,
        "        for (std::size_t word = 0; word < element_words; ++word) {",
    );
    push_line(
        output,
        "            words[element * element_words + word] = raw_value[native_element][word];",
    );
    push_line(output, "        }");
    if wide.requires_final_word_mask() {
        push_line(
            output,
            &format!(
                "        words[element * element_words + {}] &= {};",
                wide.word_count().saturating_sub(1),
                wide.final_word_mask_literal()
            ),
        );
    }
    push_line(output, "    }");
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::num::NonZeroU32;

    use super::{render_getter, render_setter, render_signed_output_decoder};
    use crate::metadata::{
        BitWidth, DutMetadata, PackedScalarShape, Port, PortDirection, PortShape,
    };

    fn width(value: u32) -> Result<BitWidth, io::Error> {
        NonZeroU32::new(value).map(BitWidth::new).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "test width must be non-zero")
        })
    }

    fn port(direction: PortDirection, bits: u32, signed: bool) -> Result<Port, io::Error> {
        let width = width(bits)?;

        Ok(Port {
            name: String::from("value"),
            direction,
            width,
            signed,
            shape: PortShape::PackedScalar(PackedScalarShape { width, signed }),
        })
    }

    #[test]
    fn signed_setter_converts_to_unsigned_before_masking() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut output = String::new();
        let port = port(PortDirection::Input, 5, true)?;

        render_setter(&mut output, &port, "set_value", "value", "Dut");

        assert!(output.contains("decltype(impl_->model->value)>"));
        assert!(output.contains("impl_->model->value = raw_value;"));
        assert!(!output.contains("impl_->model->value()"));
        assert!(!output.contains("impl_->model->value(raw_value);"));

        assert!(output.contains("using UnsignedType = std::uint8_t;"));

        assert!(output.contains("const auto bits = static_cast<UnsignedType>(value);"));

        assert!(output.contains("bits & static_cast<std::uint8_t>(0x1FULL)"));

        assert!(!output.contains("value &"));

        Ok(())
    }

    #[test]
    fn full_width_signed_setter_uses_all_bits() -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        let port = port(PortDirection::Input, 64, true)?;

        render_setter(&mut output, &port, "set_value", "value", "Dut");

        assert!(output.contains("using UnsignedType = std::uint64_t;"));

        assert!(output.contains("RawType raw_value{static_cast<RawType>(bits)};"));

        Ok(())
    }

    #[test]
    fn signed_getter_uses_declared_width() -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        let port = port(PortDirection::Output, 5, true)?;

        render_getter(&mut output, &port, "value", "value", "Dut");

        assert!(output.contains("Impl::sign_extend<std::int8_t, 5>"));
        assert!(output.contains("impl_->model->value\n"));
        assert!(!output.contains("impl_->model->value()"));

        Ok(())
    }

    #[test]
    fn signed_sixty_four_bit_getter_uses_sign_extension() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut output = String::new();
        let port = port(PortDirection::Output, 64, true)?;

        render_getter(&mut output, &port, "value", "value", "Dut");

        assert!(output.contains("Impl::sign_extend<std::int64_t, 64>"));

        Ok(())
    }

    #[test]
    fn emits_signed_decoder_only_for_signed_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let signed = DutMetadata {
            name: String::from("signed"),
            top_module: String::from("signed"),
            ports: vec![port(PortDirection::Output, 5, true)?],
        };

        let unsigned = DutMetadata {
            name: String::from("unsigned"),
            top_module: String::from("unsigned"),
            ports: vec![port(PortDirection::Output, 5, false)?],
        };

        let mut signed_output = String::new();
        let mut unsigned_output = String::new();

        render_signed_output_decoder(&mut signed_output, &signed);

        render_signed_output_decoder(&mut unsigned_output, &unsigned);

        assert!(signed_output.contains("static constexpr Signed sign_extend"));

        assert!(signed_output.contains("if constexpr (Width < storage_width)"));

        assert!(signed_output.contains("std::numeric_limits<Signed>::min()"));

        assert!(unsigned_output.is_empty());

        Ok(())
    }

    #[test]
    fn wide_input_setter_emits_three_word_transfer_and_top_mask()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        let port = port(PortDirection::Input, 65, false)?;

        render_setter(&mut output, &port, "set_value", "value", "Dut");

        assert!(output.contains("constexpr std::size_t expected_words{3};"));
        assert!(output.contains("if (words.size() != expected_words) {"));
        assert!(output.contains("raw_value[index] = words[index];"));
        assert!(output.contains("raw_value[2] &= 0x1U;"));
        assert!(output.contains("decltype(\n            impl_->model->value\n"));
        assert!(output.contains("impl_->model->value = raw_value;"));
        assert!(!output.contains("impl_->model->value(raw_value);"));
        assert!(output.contains("return false;"));
        assert!(output.contains("return true;"));

        Ok(())
    }

    #[test]
    fn wide_output_getter_omits_redundant_full_word_mask() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut output = String::new();
        let port = port(PortDirection::Output, 96, false)?;

        render_getter(&mut output, &port, "value", "value", "Dut");

        assert!(output.contains("constexpr std::size_t expected_words{3};"));
        assert!(output.contains("words[index] = raw_value[index];"));
        assert!(!output.contains("words[2] &= 0xFFFFFFFFU;"));
        assert!(output.contains("const auto& raw_value =\n        impl_->model->value;"));
        assert!(!output.contains("impl_->model->value()"));

        Ok(())
    }

    #[test]
    fn signed_wide_input_uses_same_transfer_logic_as_unsigned()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        let port = port(PortDirection::Input, 129, true)?;

        render_setter(&mut output, &port, "set_value", "value", "Dut");

        assert!(output.contains("constexpr std::size_t expected_words{5};"));
        assert!(output.contains("raw_value[index] = words[index];"));
        assert!(output.contains("raw_value[4] &= 0x1U;"));
        assert!(!output.contains("UnsignedType"));

        Ok(())
    }

    #[test]
    fn wide_getter_rejects_invalid_slice_length_before_indexing()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut output = String::new();
        let port = port(PortDirection::Output, 129, true)?;

        render_getter(&mut output, &port, "value", "value", "Dut");

        let guard = output
            .find("if (words.size() != expected_words) {")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing size guard"))?;
        let index = output
            .find("words[index] = raw_value[index];")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing copy loop"))?;

        assert!(guard < index);
        assert!(output.contains("return false;"));

        Ok(())
    }
}
