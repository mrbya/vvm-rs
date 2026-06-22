//! C++ adapter source generation.

use super::names::DutNames;
use super::types::SignalType;
use crate::metadata::{DutMetadata, Port, PortDirection};

/// Complete generated C++ adapter text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CppAdapterText {
    /// Generated public header.
    pub(super) header: String,

    /// Generated implementation source.
    pub(super) source: String,
}

/// Renders a complete C++ adapter.
pub(super) fn render(metadata: &DutMetadata, names: &DutNames) -> CppAdapterText {
    CppAdapterText {
        header: render_header(metadata, names),
        source: render_source(metadata, names),
    }
}

/// Renders the generated public adapter header.
fn render_header(metadata: &DutMetadata, names: &DutNames) -> String {
    let mut output = String::new();

    push_line(&mut output, "#pragma once");
    push_line(&mut output, "");
    push_line(&mut output, "#include <cstdint>");
    push_line(&mut output, "#include <memory>");
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("namespace vvm::{} {{", names.namespace),
    );
    push_line(&mut output, "");
    push_line(&mut output, &format!("class {} final {{", names.cpp_type));
    push_line(&mut output, "public:");
    push_line(&mut output, &format!("    {}();", names.cpp_type));
    push_line(&mut output, &format!("    ~{}() noexcept;", names.cpp_type));
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!(
            "    {}(const {}&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(
        &mut output,
        &format!(
            "    {}& operator=(const {}&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("    {}({}&&) = delete;", names.cpp_type, names.cpp_type),
    );
    push_line(
        &mut output,
        &format!(
            "    {}& operator=({}&&) = delete;",
            names.cpp_type, names.cpp_type
        ),
    );
    push_line(&mut output, "");
    push_line(&mut output, "    void eval() noexcept;");
    push_line(&mut output, "    void finish() noexcept;");
    push_line(&mut output, "");

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        let signal_type = SignalType::from_width(port.width);

        match port.direction {
            PortDirection::Input => {
                push_line(
                    &mut output,
                    &format!(
                        "    void {}({} value) noexcept;",
                        port_names.method,
                        signal_type.cpp_type()
                    ),
                );
            }
            PortDirection::Output => {
                push_line(
                    &mut output,
                    &format!(
                        "    [[nodiscard]] {} {}() const noexcept;",
                        signal_type.cpp_type(),
                        port_names.method
                    ),
                );
            }
            PortDirection::Inout => {}
        }
    }

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

/// Renders the generated adapter implementation.
fn render_source(metadata: &DutMetadata, names: &DutNames) -> String {
    let mut output = String::new();

    push_line(
        &mut output,
        &format!("#include \"{}.hpp\"", names.file_stem),
    );
    push_line(&mut output, "");
    push_line(&mut output, &format!("#include \"{}.h\"", names.model_type));
    push_line(&mut output, "#include \"verilated.h\"");
    push_line(&mut output, "");
    push_line(&mut output, "#include <cstdint>");
    push_line(&mut output, "#include <memory>");
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("namespace vvm::{} {{", names.namespace),
    );
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("class {}::Impl final {{", names.cpp_type),
    );
    push_line(&mut output, "public:");
    push_line(&mut output, "    Impl()");
    push_line(
        &mut output,
        "        : context{std::make_unique<VerilatedContext>()},",
    );
    push_line(
        &mut output,
        &format!(
            "          model{{std::make_unique<{}>(context.get())}} {{",
            names.model_type
        ),
    );

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        if port.direction != PortDirection::Input {
            continue;
        }

        push_line(&mut output, "        {");
        push_line(
            &mut output,
            &format!(
                "            auto raw_value = static_cast<decltype(model->{}())>(0);",
                port_names.accessor
            ),
        );
        push_line(
            &mut output,
            &format!("            model->{}(raw_value);", port_names.accessor),
        );
        push_line(&mut output, "        }");
    }

    push_line(&mut output, "    }");
    push_line(&mut output, "");
    push_line(
        &mut output,
        "    std::unique_ptr<VerilatedContext> context;",
    );
    push_line(
        &mut output,
        &format!("    std::unique_ptr<{}> model;", names.model_type),
    );
    push_line(&mut output, "    bool finished{false};");
    push_line(&mut output, "};");
    push_line(&mut output, "");

    push_line(
        &mut output,
        &format!("{}::{}()", names.cpp_type, names.cpp_type),
    );
    push_line(&mut output, "    : impl_{std::make_unique<Impl>()} {}");
    push_line(&mut output, "");

    push_line(
        &mut output,
        &format!("{}::~{}() noexcept {{", names.cpp_type, names.cpp_type),
    );
    push_line(&mut output, "    finish();");
    push_line(&mut output, "}");
    push_line(&mut output, "");

    push_line(
        &mut output,
        &format!("void {}::eval() noexcept {{", names.cpp_type),
    );
    push_line(&mut output, "    if (!impl_->finished) {");
    push_line(&mut output, "        impl_->model->eval();");
    push_line(&mut output, "    }");
    push_line(&mut output, "}");
    push_line(&mut output, "");

    push_line(
        &mut output,
        &format!("void {}::finish() noexcept {{", names.cpp_type),
    );
    push_line(&mut output, "    if (impl_->finished) {");
    push_line(&mut output, "        return;");
    push_line(&mut output, "    }");
    push_line(&mut output, "");
    push_line(&mut output, "    impl_->finished = true;");
    push_line(&mut output, "    impl_->model->final();");
    push_line(&mut output, "}");

    for (port, port_names) in metadata.ports.iter().zip(&names.ports) {
        match port.direction {
            PortDirection::Input => {
                render_setter(
                    &mut output,
                    port,
                    &port_names.method,
                    &port_names.accessor,
                    &names.cpp_type,
                );
            }
            PortDirection::Output => {
                render_getter(
                    &mut output,
                    port,
                    &port_names.method,
                    &port_names.accessor,
                    &names.cpp_type,
                );
            }
            PortDirection::Inout => {}
        }
    }

    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!(
            "std::unique_ptr<{}> {}() noexcept {{",
            names.cpp_type, names.factory
        ),
    );
    push_line(&mut output, "    try {");
    push_line(
        &mut output,
        &format!("        return std::make_unique<{}>();", names.cpp_type),
    );
    push_line(&mut output, "    } catch (...) {");
    push_line(&mut output, "        return nullptr;");
    push_line(&mut output, "    }");
    push_line(&mut output, "}");
    push_line(&mut output, "");
    push_line(
        &mut output,
        &format!("}} // namespace vvm::{}", names.namespace),
    );

    output
}

/// Renders one generated input setter.
fn render_setter(output: &mut String, port: &Port, method: &str, accessor: &str, cpp_type: &str) {
    let signal_type = SignalType::from_width(port.width);

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
        &format!(
            "    auto raw_value = static_cast<decltype(impl_->model->{accessor}())>({value_expression});"
        ),
    );
    push_line(output, &format!("    impl_->model->{accessor}(raw_value);"));
    push_line(output, "}");
}

/// Renders one generated output getter.
fn render_getter(output: &mut String, port: &Port, method: &str, accessor: &str, cpp_type: &str) {
    let signal_type = SignalType::from_width(port.width);

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
