use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::{
    Attribute, Error, Expr, FnArg, ItemFn, Lit, LitStr, Meta, PathArguments, Result, ReturnType,
    Type,
};

use crate::test::attrs::{ReplayAttribute, TestAttributes};

/// Validated input for `#[vvm::test]` expansion.
pub(super) struct Input {
    /// Original typed test function.
    pub(super) item: ItemFn,

    /// Stable registry name.
    pub(super) name: LitStr,

    /// Human-readable description.
    pub(super) description: LitStr,

    /// Whether waveform tracing capability supported.
    pub(super) trace: bool,

    /// Whether cycle override capability supported.
    pub(super) cycles: bool,

    /// Replay capability configuration.
    pub(super) replay: ReplayAttribute,

    /// Whether the original function accepts `&TestRunConfig`.
    pub(super) accepts_config: bool,

    /// Conditional-compilation attributes copied to generated items.
    pub(super) cfg_attributes: Vec<Attribute>,
}

impl Input {
    /// Parses and validates one attributed function.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported function forms, invalid metadata, or a
    /// malformed configuration argument.
    pub(super) fn parse(attributes: TokenStream, item: ItemFn) -> Result<Self> {
        let attributes = TestAttributes::parse(attributes)?;
        validates_function_shape(&item)?;

        let accepts_config = validate_arguments(&item, attributes.configurable())?;
        let name = resolve_name(&attributes, &item)?;
        let description = resolve_description(&attributes, &item)?;
        let cfg_attributes = item
            .attrs
            .iter()
            .filter(|attribute| {
                attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr")
            })
            .cloned()
            .collect();

        Ok(Self {
            item,
            name,
            description,
            trace: attributes.trace,
            cycles: attributes.cycles,
            replay: attributes.replay,
            accepts_config,
            cfg_attributes,
        })
    }
}

/// Validates function modifiers and return syntax.
fn validates_function_shape(item: &ItemFn) -> Result<()> {
    let signature = &item.sig;

    if let Some(constness) = &signature.constness {
        return Err(Error::new_spanned(
            constness,
            "VVM tests must not be `const` functions",
        ));
    }

    if let Some(asyncness) = &signature.asyncness {
        return Err(Error::new_spanned(
            asyncness,
            "VVM tests must be synchronous functions",
        ));
    }

    if let Some(unsafety) = &signature.unsafety {
        return Err(Error::new_spanned(
            unsafety,
            "VVM tests must not be `unsafe` functions",
        ));
    }

    if let Some(abi) = &signature.abi {
        return Err(Error::new_spanned(abi, "VVM tests must not use Rust ABI"));
    }

    if let Some(variadic) = &signature.variadic {
        return Err(Error::new_spanned(
            variadic,
            "VVM tests must not be variadic",
        ));
    }

    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &signature.generics,
            "VVM tests must not be generic",
        ));
    }

    if matches!(&signature.output, ReturnType::Default) {
        return Err(Error::new_spanned(
            &signature.ident,
            "VVM tests must return a value convertible into `TestOutcome`",
        ));
    }

    Ok(())
}

/// Validates the optional `&TestRunConfig` argument.
fn validate_arguments(item: &ItemFn, configurable: bool) -> Result<bool> {
    let mut inputs = item.sig.inputs.iter();
    let first = inputs.next();

    if inputs.next().is_some() {
        return Err(Error::new_spanned(
            &item.sig.inputs,
            "VVM tests accept at most one `&TestRunConfig` argument",
        ));
    }

    let Some(argument) = first else {
        if configurable {
            return Err(Error::new_spanned(
                &item.sig.ident,
                "tests with configurable capabilities must accept `&TestRunConfig`",
            ));
        }

        return Ok(false);
    };

    let FnArg::Typed(argument) = argument else {
        return Err(Error::new_spanned(
            argument,
            "VVM test methods with a `self` receiver are not supported",
        ));
    };

    let Type::Reference(reference) = argument.ty.as_ref() else {
        return Err(Error::new_spanned(
            &argument.ty,
            "VVM test configuration must be passed as `&TestRunConfig`",
        ));
    };

    if reference.mutability.is_some() {
        return Err(Error::new_spanned(
            reference,
            "VVM test configuration must be an immutable `&TestRunConfig` reference",
        ));
    }

    let Type::Path(path) = reference.elem.as_ref() else {
        return Err(Error::new_spanned(
            &reference.elem,
            "VVM test configuration must be `&TestRunConfig`",
        ));
    };

    let valid = path.qself.is_none()
        && path.path.segments.last().is_some_and(|segment| {
            segment.ident == "TestRunConfig" && matches!(&segment.arguments, PathArguments::None)
        });

    if !valid {
        return Err(Error::new_spanned(
            &reference.elem,
            "VVM test configuration must be `&TestRunConfig`",
        ));
    }

    Ok(true)
}

/// Resolves and validates the registry name.
fn resolve_name(attributes: &TestAttributes, item: &ItemFn) -> Result<LitStr> {
    let name = attributes.name.clone().unwrap_or_else(|| {
        LitStr::new(
            &item.sig.ident.unraw().to_string().replace('_', "-"),
            item.sig.ident.span(),
        )
    });

    if valid_test_name(&name.value()) {
        return Ok(name);
    }

    Err(Error::new(
        name.span(),
        format!(
            "invalid VVM test name `{}`; names must begin with an ASCII lowercase letter and \
             contain only lowercase letters, digits, `-`, `_`, or `.`",
            name.value(),
        ),
    ))
}

/// Resolves the explicit or rustdoc-derived description.
fn resolve_description(attributes: &TestAttributes, item: &ItemFn) -> Result<LitStr> {
    if let Some(description) = attributes.description.clone() {
        return Ok(description);
    }

    let description = first_doc_paragraph(&item.attrs).ok_or_else(|| {
        Error::new_spanned(
            &item.sig.ident,
            "VVM tests require a rustdoc description or `description = \"...\"`",
        )
    })?;

    Ok(LitStr::new(&description, item.sig.ident.span()))
}

/// Returns the first nonempty rustdoc paragraph.
fn first_doc_paragraph(attributes: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    let mut started = false;

    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
    {
        let Meta::NameValue(name_value) = &attribute.meta else {
            continue;
        };

        let Expr::Lit(expression) = &name_value.value else {
            continue;
        };

        let Lit::Str(line) = &expression.lit else {
            continue;
        };

        let line = line.value();
        let line = line.trim();

        if line.is_empty() {
            if started {
                break;
            }

            continue;
        }

        started = true;
        lines.push(line.to_owned());
    }

    (!lines.is_empty()).then(|| lines.join(" "))
}

/// Returns whether a registry name follows VVM's stable name grammar.
fn valid_test_name(name: &str) -> bool {
    let mut characters = name.chars();

    let Some(first) = characters.next() else {
        return false;
    };

    first.is_ascii_lowercase()
        && characters.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_' | '.')
        })
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{ItemFn, parse_quote};

    use super::Input;

    #[test]
    fn infers_name_and_description() -> Result<(), Box<dyn std::error::Error>> {
        let item: ItemFn = parse_quote! {
            /// Randomized counter regression.
            ///
            /// Additional implementation details.
            fn counter_random(config: &vvm::TestRunConfig) -> ResultType {
                run(config)
            }
        };

        let input = Input::parse(quote!(trace, replay), item)?;

        assert_eq!(input.name.value(), "counter-random");
        assert_eq!(input.description.value(), "Randomized counter regression.");
        assert!(input.trace);
        assert!(input.replay.enabled());
        assert!(input.accepts_config);
        Ok(())
    }

    #[test]
    fn configurable_test_requires_config_argument() {
        let item: ItemFn = parse_quote! {
            /// Smoke test.
            fn smoke() -> ResultType {
                run()
            }
        };

        assert!(Input::parse(quote!(trace), item).is_err());
    }
}
