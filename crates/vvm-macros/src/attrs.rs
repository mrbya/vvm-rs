use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::{Attribute, Error, Ident, LitStr, Path, Result, Token};

/// Parsed `#[vvm(port...)]` field mapping
pub struct PortAttribute {
    /// Explicit HDL port name, or `None` for the field name.
    name: Option<LitStr>,
}

impl PortAttribute {
    /// Creates the generated DUT method identifier for this port.
    ///
    /// `prefix` is `"set_"` for `Drive` and empty for `Sample`.
    pub fn method_ident(&self, field: &Ident, prefix: &str) -> Result<Ident> {
        let (port_name, span) = self.name.as_ref().map_or_else(
            || (field.unraw().to_string(), field.span()),
            |name| (name.value(), name.span()),
        );

        port_method_ident(&port_name, span, prefix)
    }
}

/// Parses the mandatory `#[vvm(dut = path)]` attribute.
///
/// # Errors
///
/// Returns an error for missing, duplicated, malformed, or unknown container
/// options.
pub fn parse_dut_path(attributes: &[Attribute], item_span: Span) -> Result<Path> {
    let mut dut = None;

    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("vvm"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("dut") {
                return Err(meta.error("unsupported `vvm` container option: expected `dut`"));
            }

            if dut.is_some() {
                return Err(meta.error("duplicate `dut` option"));
            }

            let value = meta.value()?;
            let path: Path = value.parse()?;

            dut = Some(path);

            Ok(())
        })?;
    }

    dut.ok_or_else(|| Error::new(item_span, "missing `#[vvm(dut = path)]` attribute"))
}

/// Parses an optional `#[vvm(port)]` or
/// `#[vvm(port = "hdl_name")]` attribute.
///
/// # Errors
///
/// Returns an error for duplicated, malformed, or unknown field options.
pub fn parse_port_attribute(attributes: &[Attribute]) -> Result<Option<PortAttribute>> {
    let mut port = None;

    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("vvm"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("port") {
                return Err(meta.error("unsupported `vvm` field option: expected `port`"));
            }

            if port.is_some() {
                return Err(meta.error("duplicate `port` option"));
            }

            let name = if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let name: LitStr = value.parse()?;

                if name.value().is_empty() {
                    return Err(Error::new(name.span(), "port name must not be empty"));
                }

                Some(name)
            } else {
                None
            };

            port = Some(PortAttribute { name });

            Ok(())
        })?;
    }

    Ok(port)
}

/// Creates a Rust method identifier from an HDL port name.
///
/// # Errors
///
/// Returns an error when the port name and prefix do not form a valid Rust
/// identifier.
pub fn port_method_ident(port_name: &str, span: Span, prefix: &str) -> Result<Ident> {
    let method_name = format!("{prefix}{port_name}");

    let mut method = syn::parse_str::<Ident>(&method_name).map_err(|_parse_error| {
        Error::new(
            span,
            format!("port name `{port_name}` does not produce a valid Rust method identifier"),
        )
    })?;

    method.set_span(span);

    Ok(method)
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{Attribute, parse_quote};

    use super::{parse_dut_path, parse_port_attribute, port_method_ident};

    #[test]
    fn parses_dut_path_and_port_mappings() -> Result<(), Box<dyn std::error::Error>> {
        let attributes = vec![parse_quote!(#[vvm(dut = crate::models::Counter)])];
        let raw_field: syn::Field = parse_quote!(#[vvm(port)] raw_status: bool);
        let data_field: syn::Field = parse_quote!(#[vvm(port = "data_out")] data: u8);
        let dut = parse_dut_path(&attributes, proc_macro2::Span::call_site())?;
        let raw_port = parse_port_attribute(&raw_field.attrs)?.expect("port attribute exists");
        let data_port = parse_port_attribute(&data_field.attrs)?.expect("port attribute exists");

        assert_eq!(quote!(#dut).to_string(), "crate :: models :: Counter");
        assert_eq!(
            raw_port.method_ident(&parse_quote!(raw_status), "")?,
            "raw_status"
        );
        assert_eq!(
            data_port.method_ident(&parse_quote!(data), "set_")?,
            "set_data_out"
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_dut_attributes_with_specific_diagnostics() {
        let cases: [(Vec<Attribute>, &str); 4] = [
            (Vec::new(), "missing `#[vvm(dut = path)]` attribute"),
            (
                vec![parse_quote!(#[vvm(clock = "clk")])],
                "unsupported `vvm` container option: expected `dut`",
            ),
            (
                vec![parse_quote!(#[vvm(dut = A, dut = B)])],
                "duplicate `dut` option",
            ),
            (vec![parse_quote!(#[vvm(dut)])], "expected `=`"),
        ];

        for (attributes, expected) in cases {
            let result = parse_dut_path(&attributes, proc_macro2::Span::call_site());

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn rejects_invalid_port_attributes_with_specific_diagnostics() {
        let cases: [(Vec<Attribute>, &str); 4] = [
            (
                vec![parse_quote!(#[vvm(signal)])],
                "unsupported `vvm` field option: expected `port`",
            ),
            (
                vec![parse_quote!(#[vvm(port, port)])],
                "duplicate `port` option",
            ),
            (
                vec![parse_quote!(#[vvm(port = "")])],
                "port name must not be empty",
            ),
            (
                vec![parse_quote!(#[vvm(port = identifier)])],
                "expected string literal",
            ),
        ];

        for (attributes, expected) in cases {
            let result = parse_port_attribute(&attributes);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn rejects_port_names_that_cannot_form_method_identifiers() {
        let result = port_method_ident("not-a-port", proc_macro2::Span::call_site(), "set_");

        assert_eq!(
            result.as_ref().err().map(ToString::to_string).as_deref(),
            Some("port name `not-a-port` does not produce a valid Rust method identifier")
        );
    }
}
