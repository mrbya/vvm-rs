use proc_macro2::TokenStream;
use syn::parse::Parser;
use syn::{Expr, LitStr, Result, Token};

/// Parsed replay capability configuration.
#[derive(Default)]
pub(super) enum ReplayAttribute {
    /// Test does not supported randomized replay.
    #[default]
    Disabled,

    /// Test supports replay, optionally with a default token.
    Enabled {
        /// Default replay ex[ression.]
        default: Box<Option<Expr>>,
    },
}

impl ReplayAttribute {
    /// Returns whether replay support was requested.
    pub const fn enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }

    /// Returns the configured default replay token expression, if any.
    pub(super) fn default_expr(&self) -> Option<&Expr> {
        match *self {
            Self::Disabled => None,
            Self::Enabled { ref default } => default.as_ref().as_ref(),
        }
    }
}

/// Parsed `#[vvm::test(...)]` attributes.
#[derive(Default)]
pub(super) struct TestAttributes {
    /// Explicit test name.
    pub name: Option<LitStr>,

    /// Explicit human-readable description.
    pub description: Option<LitStr>,

    /// Waveform tracing capability configuration.
    pub trace: bool,

    /// Cycle override capability configuration.
    pub cycles: bool,

    /// Replay capability configuration.
    pub replay: ReplayAttribute,
    /// Functional coverage capture declaration.
    pub coverage: bool,
}

impl TestAttributes {
    /// Parses attribute arguments.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown, duplicated, or malformed options.
    pub(super) fn parse(tokens: TokenStream) -> Result<Self> {
        let mut attributes = Self::default();
        let mut replay_seen = false;
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                if attributes.name.is_some() {
                    return Err(meta.error("duplicate `name` option"));
                }
                let value = meta.value()?;
                let name: LitStr = value.parse()?;

                if name.value().is_empty() {
                    return Err(syn::Error::new(name.span(), "test name must not be empty"));
                }

                attributes.name = Some(name);
                return Ok(());
            }

            if meta.path.is_ident("description") {
                if attributes.description.is_some() {
                    return Err(meta.error("duplicate `description` option"));
                }

                let value = meta.value()?;
                let description: LitStr = value.parse()?;

                if description.value().trim().is_empty() {
                    return Err(syn::Error::new(
                        description.span(),
                        "test description must not be empty",
                    ));
                }

                attributes.description = Some(description);
                return Ok(());
            }

            if meta.path.is_ident("trace") {
                if attributes.trace {
                    return Err(meta.error("duplicate `trace` option"));
                }

                if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
                    return Err(meta.error("`trace` does not accept a value"));
                }

                attributes.trace = true;
                return Ok(());
            }

            if meta.path.is_ident("cycles") {
                if attributes.cycles {
                    return Err(meta.error("duplicate `cycles` option"));
                }

                if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
                    return Err(meta.error("`cycles` does not accept a value"));
                }

                attributes.cycles = true;
                return Ok(());
            }

            if meta.path.is_ident("replay") {
                if replay_seen {
                    return Err(meta.error("duplicate `replay` option"));
                }

                replay_seen = true;

                if meta.input.peek(Token![=]) {
                    return Err(meta.error("use `replay` or `replay(default = expression)`"));
                }

                let mut default = None;
                let has_nested_options = meta.input.peek(syn::token::Paren);

                if has_nested_options {
                    meta.parse_nested_meta(|nested| {
                        if !nested.path.is_ident("default") {
                            return Err(nested.error(
                                "unsupported `replay` option: expected `default = expression`",
                            ));
                        }

                        if default.is_some() {
                            return Err(nested.error("duplicate `default` replay option"));
                        }

                        let value = nested.value()?;
                        default = Some(value.parse::<Expr>()?);
                        Ok(())
                    })?;

                    if default.is_none() {
                        return Err(meta
                            .error("empty `replay(...)` option: expected `default = expression`"));
                    }
                }

                attributes.replay = ReplayAttribute::Enabled {
                    default: Box::new(default),
                };
                return Ok(());
            }

            if meta.path.is_ident("coverage") {
                return parse_coverage_option(&meta, &mut attributes.coverage);
            }

            Err(meta.error(
                "unsupported `vvm::test` option: expected `name`, `description`, `trace`, \
                 `cycles`, `replay`, or `coverage`",
            ))
        });

        parser.parse2(tokens)?;
        Ok(attributes)
    }

    /// Returns whether the test exposes any runtime-configurable capability.
    pub(super) const fn configurable(&self) -> bool {
        self.trace || self.cycles || self.replay.enabled() || self.coverage
    }
}

/// Parses the flag-only coverage test capability.
fn parse_coverage_option(meta: &syn::meta::ParseNestedMeta<'_>, coverage: &mut bool) -> Result<()> {
    if *coverage {
        return Err(meta.error("duplicate `coverage` option"));
    }

    if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
        return Err(meta.error("`coverage` does not accept a value"));
    }

    *coverage = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::TestAttributes;

    #[test]
    fn parses_all_supported_options() -> Result<(), Box<dyn std::error::Error>> {
        let attributes = TestAttributes::parse(quote! {
            name = "counter-smoke",
            description = "Checks the counter.",
            trace,
            cycles,
            replay(default = seed + 1),
            coverage
        })?;

        assert_eq!(
            attributes.name.as_ref().map(syn::LitStr::value),
            Some("counter-smoke".into())
        );
        assert_eq!(
            attributes.description.as_ref().map(syn::LitStr::value),
            Some("Checks the counter.".into())
        );
        assert!(attributes.trace);
        assert!(attributes.cycles);
        assert!(attributes.replay.enabled());
        assert_eq!(
            attributes
                .replay
                .default_expr()
                .map(quote::ToTokens::to_token_stream)
                .map(|tokens| tokens.to_string()),
            Some("seed + 1".into())
        );
        assert!(attributes.coverage);
        assert!(attributes.configurable());
        Ok(())
    }

    #[test]
    fn rejects_invalid_options_with_specific_diagnostics() {
        let cases = [
            (quote!(name = "",), "test name must not be empty"),
            (
                quote!(description = " \t ",),
                "test description must not be empty",
            ),
            (quote!(trace, trace), "duplicate `trace` option"),
            (quote!(cycles = 4), "`cycles` does not accept a value"),
            (
                quote!(replay = 3),
                "use `replay` or `replay(default = expression)`",
            ),
            (
                quote!(replay()),
                "unexpected end of input, expected nested attribute",
            ),
            (
                quote!(replay(default = 1, default = 2)),
                "duplicate `default` replay option",
            ),
            (quote!(coverage, coverage), "duplicate `coverage` option"),
            (
                quote!(unknown),
                "unsupported `vvm::test` option: expected `name`, `description`, `trace`, \
                 `cycles`, `replay`, or `coverage`",
            ),
        ];

        for (tokens, expected) in cases {
            let result = TestAttributes::parse(tokens);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }
}
