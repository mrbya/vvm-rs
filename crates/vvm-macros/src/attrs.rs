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
