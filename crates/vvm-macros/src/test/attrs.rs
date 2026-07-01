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
        default: Option<Expr>,
    },
}

impl ReplayAttribute {
    /// Returns whether replay support was requested.
    pub const fn enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
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

                attributes.replay = ReplayAttribute::Enabled { default };
                return Ok(());
            }

            Err(meta.error(
                "unsupported `vvm::test` option: expected `name`, `description`, `trace`, \
                 `cycles`, or `replay`",
            ))
        });

        parser.parse2(tokens)?;
        Ok(attributes)
    }

    /// Returns whether the test exposes any runtime-configurable capability.
    pub(super) const fn configurable(&self) -> bool {
        self.trace || self.cycles || self.replay.enabled()
    }
}
