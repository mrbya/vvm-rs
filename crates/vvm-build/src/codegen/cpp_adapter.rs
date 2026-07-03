use super::names::DutNames;
use super::types::SignalType;
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
) -> CppAdapterText {
    let trace = trace.filter(|options| options.format == TraceFormat::Vcd);

    CppAdapterText {
        header: render_header(metadata, names, trace.is_some()),
        source: render_source(metadata, names, trace),
    }
}

/// Renders the generated public adapter header.
fn render_header(metadata: &DutMetadata, names: &DutNames, traced: bool) -> String {
    let mut output = String::new();

    render_header_prelude(&mut output, names, traced);
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
fn render_header_prelude(output: &mut String, names: &DutNames, traced: bool) {
    push_line(output, GENERATED_NOTICE);
    push_line(output, "");
    push_line(output, "#pragma once");
    push_line(output, "");
    if traced {
        push_line(output, "#include \"rust/cxx.h\"");
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
        let signal_type = SignalType::from_port(port);

        match port.direction {
            PortDirection::Input => push_line(
                output,
                &format!(
                    "    void {}({} value) noexcept;",
                    port_names.method,
                    signal_type.cpp_type()
                ),
            ),
            PortDirection::Output => push_line(
                output,
                &format!(
                    "    [[nodiscard]] {} {}() const noexcept;",
                    signal_type.cpp_type(),
                    port_names.method
                ),
            ),
            PortDirection::Inout => {}
        }
    }
}

/// Renders the generated adapter implementation.
fn render_source(metadata: &DutMetadata, names: &DutNames, trace: Option<TraceOptions>) -> String {
    let mut output = String::new();
    let traced = trace.is_some();
    let trace_depth = trace.map(|options| options.depth);

    render_source_prelude(&mut output, names, traced);
    render_impl_class(&mut output, metadata, names, trace);
    render_lifecycle_methods(&mut output, names, trace_depth);
    render_port_methods(&mut output, metadata, names);
    render_time_methods(&mut output, names);
    render_factory_function(&mut output, names);
    render_source_epilogue(&mut output, names);

    output
}

/// Renders the source-file prelude and namespace opening.
fn render_source_prelude(output: &mut String, names: &DutNames, traced: bool) {
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
                "            using RawType = std::decay_t<decltype(model->{}())>;",
                port_names.accessor
            ),
        );
        push_line(output, "            RawType raw_value{0};");
        push_line(
            output,
            &format!("            model->{}(raw_value);", port_names.accessor),
        );
        push_line(output, "        }");
    }
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
                    &port_names.accessor,
                    &names.cpp_type,
                );
            }
            PortDirection::Output => {
                render_getter(
                    output,
                    port,
                    &port_names.method,
                    &port_names.accessor,
                    &names.cpp_type,
                );
            }
            PortDirection::Inout => {}
        }
    }
}

/// Renders the adapter timing methods.
fn render_time_methods(output: &mut String, names: &DutNames) {
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
fn render_setter(output: &mut String, port: &Port, method: &str, accessor: &str, cpp_type: &str) {
    let signal_type = SignalType::from_port(port);

    let value_expression = signal_type
        .mask_literal(port.width)
        .map_or_else(|| "value".to_owned(), |mask| format!("value & {mask}"));

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
        &format!("    using RawType = std::decay_t<decltype(impl_->model->{accessor}())>;"),
    );
    push_line(output, "");
    push_line(
        output,
        &format!("    RawType raw_value{{static_cast<RawType>({value_expression})}};"),
    );
    push_line(output, &format!("    impl_->model->{accessor}(raw_value);"));
    push_line(output, "}");
}

/// Renders one generated output getter.
fn render_getter(output: &mut String, port: &Port, method: &str, accessor: &str, cpp_type: &str) {
    let signal_type = SignalType::from_port(port);

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
            &format!("    return impl_->model->{accessor}() != 0;"),
        );
    } else if let Some(mask) = signal_type.mask_literal(port.width) {
        push_line(
            output,
            &format!(
                "    return static_cast<{}>(impl_->model->{accessor}() & {mask});",
                signal_type.cpp_type()
            ),
        );
    } else {
        push_line(
            output,
            &format!(
                "    return static_cast<{}>(impl_->model->{accessor}());",
                signal_type.cpp_type()
            ),
        );
    }

    push_line(output, "}");
}

/// Appends one line and a Unix newline.
fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
