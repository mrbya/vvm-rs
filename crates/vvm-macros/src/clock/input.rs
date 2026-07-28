use syn::{Data, DeriveInput, Error, Fields, Generics, Ident, LitStr, Path, Result};

use crate::attrs::port_method_ident;

/// Clock edge selected by the derive attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    /// Inactive low, active high.
    Rising,

    /// Inactive high, active low.
    Falling,
}

impl Edge {
    /// Parses an edge attribute.
    fn parse(value: &LitStr) -> Result<Self> {
        match value.value().as_str() {
            "rising" => Ok(Self::Rising),
            "falling" => Ok(Self::Falling),
            other => Err(Error::new(
                value.span(),
                format!("unsupported clock edge `{other}`; expected `rising` or `falling`"),
            )),
        }
    }

    /// Returns inactive and active clock levels.
    const fn levels(self) -> (bool, bool) {
        match self {
            Self::Rising => (false, true),
            Self::Falling => (true, false),
        }
    }
}

/// Parsed `Clock` implementation input.
pub(super) struct Input {
    /// Clock driver type identifier.
    pub(super) ident: Ident,

    /// Clock driver generics.
    pub(super) generics: Generics,

    /// DUT type path.
    pub(super) dut: Path,

    /// Generated clock setter.
    pub(super) setter: Ident,

    /// Inactive clock level.
    pub(super) inactive: bool,

    /// Active clock level.
    pub(super) active: bool,
}

impl Input {
    /// Parses a complete derive input.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported data shapes or invalid VVM metadata.
    pub(super) fn parse(input: DeriveInput) -> Result<Self> {
        ensure_unit_struct(&input)?;

        let item_span = input.ident.span();

        let mut dut = None;
        let mut clock = None;
        let mut edge = None;

        for attribute in input
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("vvm"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("dut") {
                    if dut.is_some() {
                        return Err(meta.error("duplicate `dut` option"));
                    }

                    let value = meta.value()?;
                    let path: Path = value.parse()?;

                    dut = Some(path);

                    return Ok(());
                }

                if meta.path.is_ident("clock") {
                    if clock.is_some() {
                        return Err(meta.error("duplicate `clock` option"));
                    }

                    let value = meta.value()?;
                    let name: LitStr = value.parse()?;

                    if name.value().is_empty() {
                        return Err(Error::new(name.span(), "clock port name must not be empty"));
                    }

                    clock = Some(name);

                    return Ok(());
                }

                if meta.path.is_ident("edge") {
                    if edge.is_some() {
                        return Err(meta.error("duplicate `edge` option"));
                    }

                    let value = meta.value()?;
                    let value: LitStr = value.parse()?;

                    edge = Some(Edge::parse(&value)?);

                    return Ok(());
                }

                Err(meta
                    .error("unsupported `vvm` clock option; expected `dut`, `clock`, or `edge`"))
            })?;
        }

        let dut =
            dut.ok_or_else(|| Error::new(item_span, "missing `dut` option in `#[vvm(...)]`"))?;

        let clock = clock
            .ok_or_else(|| Error::new(item_span, "missing `clock` option in `#[vvm(...)]`"))?;

        let setter = port_method_ident(&clock.value(), clock.span(), "set_")?;

        let (inactive, active) = edge.unwrap_or(Edge::Rising).levels();

        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            dut,
            setter,
            inactive,
            active,
        })
    }
}

/// Verifies that `Clock` is derived for a unit struct.
fn ensure_unit_struct(input: &DeriveInput) -> Result<()> {
    match input.data {
        Data::Struct(ref data) if matches!(data.fields, Fields::Unit) => Ok(()),

        Data::Struct(_) | Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            &input.ident,
            "`Clock` can only be derived for a unit struct",
        )),
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{DeriveInput, parse2};

    use super::Input;

    #[test]
    fn parses_clock_metadata_and_edge_levels() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            #[vvm(dut = crate::Dut, clock = "clk_n", edge = "falling")]
            struct Clock<T>(std::marker::PhantomData<T>);
        })?;

        let result = Input::parse(input);

        assert_eq!(
            result.as_ref().err().map(ToString::to_string).as_deref(),
            Some("`Clock` can only be derived for a unit struct")
        );

        let valid_input: DeriveInput = parse2(quote! {
            #[vvm(dut = crate::Dut, clock = "clk_n", edge = "falling")]
            struct Clock;
        })?;
        let parsed = Input::parse(valid_input)?;
        let dut = &parsed.dut;

        assert_eq!(parsed.ident, "Clock");
        assert_eq!(quote!(#dut).to_string(), "crate :: Dut");
        assert_eq!(parsed.setter, "set_clk_n");
        assert_eq!((parsed.inactive, parsed.active), (true, false));
        Ok(())
    }

    #[test]
    fn rejects_invalid_clock_inputs_with_specific_diagnostics()
    -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                quote!(
                    #[vvm(clock = "clk")]
                    struct Clock;
                ),
                "missing `dut` option in `#[vvm(...)]`",
            ),
            (
                quote!(
                    #[vvm(dut = Dut)]
                    struct Clock;
                ),
                "missing `clock` option in `#[vvm(...)]`",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "")]
                    struct Clock;
                ),
                "clock port name must not be empty",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "clk", edge = "both")]
                    struct Clock;
                ),
                "unsupported clock edge `both`; expected `rising` or `falling`",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "clk", clock = "other")]
                    struct Clock;
                ),
                "duplicate `clock` option",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "clk", edge = "rising", edge = "falling")]
                    struct Clock;
                ),
                "duplicate `edge` option",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "clk", port = "x")]
                    struct Clock;
                ),
                "unsupported `vvm` clock option; expected `dut`, `clock`, or `edge`",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "bad-port")]
                    struct Clock;
                ),
                "port name `bad-port` does not produce a valid Rust method identifier",
            ),
            (
                quote!(
                    #[vvm(dut = Dut, clock = "clk")]
                    enum Clock {
                        One,
                    }
                ),
                "`Clock` can only be derived for a unit struct",
            ),
        ];

        for (tokens, expected) in cases {
            let input: DeriveInput = parse2(tokens)?;
            let result = Input::parse(input);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }

        Ok(())
    }
}
