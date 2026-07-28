use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Error, Expr, FnArg, ItemFn, Lit, LitStr, Meta, PathArguments, Result, ReturnType,
    Token, Type, parse_quote,
};

use crate::test::attrs::{ReplayAttribute, TestAttributes};
use crate::test::names::implementation_ident;

/// Validated input for `#[vvm::test]` expansion.
pub(super) struct Input {
    /// Hidden typed implementation function.
    pub(super) implementation: ItemFn,

    /// Visible Rust test function name.
    pub(super) function: syn::Ident,

    /// Attributes preserved on the visible Rust test wrapper.
    pub(super) wrapper_attributes: Vec<Attribute>,

    /// Conditional-compilation attributes copied to helper items.
    pub(super) helper_attributes: Vec<Attribute>,

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
    /// Whether coverage capture is declared.
    pub(super) coverage: bool,

    /// Supported argument accepted by the original function.
    pub(super) argument: TestArgument,
}

/// Supported optional VVM test argument.
#[derive(Clone, Copy)]
pub(super) enum TestArgument {
    /// No argument.
    None,
    /// Immutable resolved execution configuration.
    Config,
    /// Mutable per-test execution context.
    Context,
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
        validate_function_attributes(&item.attrs)?;
        validates_function_shape(&item)?;

        let argument = validate_arguments(&item, attributes.configurable())?;
        if attributes.coverage && !matches!(argument, TestArgument::Context) {
            return Err(Error::new_spanned(
                &item.sig.inputs,
                "tests declaring `coverage` must accept `&mut TestContext`",
            ));
        }
        let name = resolve_name(&attributes, &item)?;
        let description = resolve_description(&attributes, &item)?;
        let (wrapper_attributes, implementation_attributes, helper_attributes) =
            partition_attributes(&item.attrs)?;

        let function = item.sig.ident.clone();
        let mut implementation = item;
        implementation.sig.ident = implementation_ident(&function);
        implementation.vis = parse_quote!();
        implementation.attrs = implementation_attributes;

        Ok(Self {
            implementation,
            function,
            wrapper_attributes,
            helper_attributes,
            name,
            description,
            trace: attributes.trace,
            cycles: attributes.cycles,
            replay: attributes.replay,
            coverage: attributes.coverage,
            argument,
        })
    }
}

/// Validates unsupported source-level test attributes.
fn validate_function_attributes(attributes: &[Attribute]) -> Result<()> {
    for attribute in attributes {
        reject_incompatible_attribute(attribute)?;
    }

    Ok(())
}

/// Rejects attributes that conflict with VVM's standard-test integration.
fn reject_incompatible_attribute(attribute: &Attribute) -> Result<()> {
    if attribute.path().is_ident("test") {
        return Err(Error::new_spanned(
            attribute,
            "VVM tests must not also declare `#[test]`; `#[vvm::test]` generates the Rust test \
             wrapper",
        ));
    }

    if attribute.path().is_ident("should_panic") {
        return Err(Error::new_spanned(
            attribute,
            "VVM tests do not support `#[should_panic]`; use structured VVM outcomes instead",
        ));
    }

    if !attribute.path().is_ident("cfg_attr") {
        return Ok(());
    }

    let (_, metas) = split_cfg_attr(attribute)?;

    for meta in metas {
        if meta_path_is_ident(&meta, "test") {
            return Err(Error::new_spanned(
                &meta,
                "VVM tests must not also declare `#[test]`; `#[vvm::test]` generates the Rust \
                 test wrapper",
            ));
        }

        if meta_path_is_ident(&meta, "should_panic") {
            return Err(Error::new_spanned(
                &meta,
                "VVM tests do not support `#[should_panic]`; use structured VVM outcomes instead",
            ));
        }
    }

    Ok(())
}

/// Validates function modifiers and return syntax.
fn validates_function_shape(item: &ItemFn) -> Result<()> {
    let signature = &item.sig;

    if let Some(constness) = signature.constness.as_ref() {
        return Err(Error::new_spanned(
            constness,
            "VVM tests must not be `const` functions",
        ));
    }

    if let Some(asyncness) = signature.asyncness.as_ref() {
        return Err(Error::new_spanned(
            asyncness,
            "VVM tests must be synchronous functions",
        ));
    }

    if let Some(unsafety) = signature.unsafety.as_ref() {
        return Err(Error::new_spanned(
            unsafety,
            "VVM tests must not be `unsafe` functions",
        ));
    }

    if let Some(abi) = signature.abi.as_ref() {
        return Err(Error::new_spanned(
            abi,
            "VVM tests must not declare an extern ABI",
        ));
    }

    if let Some(variadic) = signature.variadic.as_ref() {
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

/// Validates the optional VVM execution argument.
fn validate_arguments(item: &ItemFn, configurable: bool) -> Result<TestArgument> {
    let mut inputs = item.sig.inputs.iter();
    let first = inputs.next();

    if inputs.next().is_some() {
        return Err(Error::new_spanned(
            &item.sig.inputs,
            "VVM tests accept at most one `&TestRunConfig` or `&mut TestContext` argument",
        ));
    }

    let Some(argument) = first else {
        if configurable {
            return Err(Error::new_spanned(
                &item.sig.ident,
                "tests with configurable capabilities must accept `&TestRunConfig` or `&mut \
                 TestContext`",
            ));
        }

        return Ok(TestArgument::None);
    };

    let argument = match *argument {
        FnArg::Typed(ref argument) => argument,
        FnArg::Receiver(ref receiver) => {
            return Err(Error::new_spanned(
                receiver,
                "VVM test methods with a `self` receiver are not supported",
            ));
        }
    };

    let Type::Reference(ref reference) = *argument.ty else {
        return Err(Error::new_spanned(
            &argument.ty,
            "VVM tests accept `&TestRunConfig` or `&mut TestContext`",
        ));
    };

    let Type::Path(ref path) = *reference.elem else {
        return Err(Error::new_spanned(
            &reference.elem,
            "VVM tests accept `&TestRunConfig` or `&mut TestContext`",
        ));
    };

    let type_name = path
        .path
        .segments
        .last()
        .filter(|segment| path.qself.is_none() && matches!(&segment.arguments, PathArguments::None))
        .map(|segment| &segment.ident);

    match (reference.mutability.is_some(), type_name) {
        (false, Some(name)) if name == "TestRunConfig" => Ok(TestArgument::Config),
        (true, Some(name)) if name == "TestContext" => Ok(TestArgument::Context),
        (false, Some(name)) if name == "TestContext" => Err(Error::new_spanned(
            reference,
            "VVM test context must be passed as mutable `&mut TestContext`",
        )),
        (true, Some(name)) if name == "TestRunConfig" => Err(Error::new_spanned(
            reference,
            "VVM test configuration must be an immutable `&TestRunConfig` reference",
        )),
        _ => Err(Error::new_spanned(
            &reference.elem,
            "VVM tests accept `&TestRunConfig` or `&mut TestContext`",
        )),
    }
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

/// Partitions source attributes between the visible wrapper and hidden helpers.
fn partition_attributes(
    attributes: &[Attribute],
) -> Result<(Vec<Attribute>, Vec<Attribute>, Vec<Attribute>)> {
    let mut wrapper = Vec::new();
    let mut implementation = Vec::new();
    let mut helper = Vec::new();

    for attribute in attributes {
        wrapper.push(attribute.clone());

        if attribute.path().is_ident("cfg") {
            implementation.push(attribute.clone());
            helper.push(attribute.clone());
            continue;
        }

        if is_lint_attribute(attribute) {
            implementation.push(attribute.clone());
            continue;
        }

        if attribute.path().is_ident("cfg_attr") {
            let implementation_attribute =
                filtered_cfg_attr(attribute, meta_is_cfg_or_lint_attribute)?;
            let helper_attribute = filtered_cfg_attr(attribute, meta_is_cfg_attribute)?;

            if let Some(filtered_attribute) = implementation_attribute {
                implementation.push(filtered_attribute);
            }

            if let Some(filtered_attribute) = helper_attribute {
                helper.push(filtered_attribute);
            }
        }
    }

    Ok((wrapper, implementation, helper))
}

/// Returns the first nonempty rustdoc paragraph.
fn first_doc_paragraph(attributes: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    let mut started = false;

    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
    {
        let Meta::NameValue(name_value) = attribute.meta.clone() else {
            continue;
        };

        let Expr::Lit(expression) = name_value.value else {
            continue;
        };

        let Lit::Str(line) = expression.lit else {
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

/// Returns whether this is one of Rust's lint-control attributes.
fn is_lint_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("allow")
        || attribute.path().is_ident("warn")
        || attribute.path().is_ident("deny")
        || attribute.path().is_ident("forbid")
        || attribute.path().is_ident("expect")
}

/// Returns whether one nested meta is a `cfg(...)` attribute.
fn meta_is_cfg_attribute(meta: &Meta) -> bool {
    meta_path_is_ident(meta, "cfg")
}

/// Returns whether one nested meta is suitable for hidden implementation items.
fn meta_is_cfg_or_lint_attribute(meta: &Meta) -> bool {
    meta_is_cfg_attribute(meta)
        || meta_path_is_ident(meta, "allow")
        || meta_path_is_ident(meta, "warn")
        || meta_path_is_ident(meta, "deny")
        || meta_path_is_ident(meta, "forbid")
        || meta_path_is_ident(meta, "expect")
}

/// Returns whether one meta path matches an identifier.
fn meta_path_is_ident(meta: &Meta, ident: &str) -> bool {
    match *meta {
        Meta::Path(ref path) => path.is_ident(ident),
        Meta::List(ref list) => list.path.is_ident(ident),
        Meta::NameValue(ref name_value) => name_value.path.is_ident(ident),
    }
}

/// Filters a `cfg_attr` to only the nested metas relevant to one target item.
fn filtered_cfg_attr(attribute: &Attribute, keep: fn(&Meta) -> bool) -> Result<Option<Attribute>> {
    let (condition, metas) = split_cfg_attr(attribute)?;
    let metas = metas.into_iter().filter(keep).collect::<Vec<_>>();

    if metas.is_empty() {
        return Ok(None);
    }

    Ok(Some(parse_quote!(#[cfg_attr(#condition, #(#metas),*)])))
}

/// Splits a `cfg_attr` into its condition tokens and nested metas.
fn split_cfg_attr(attribute: &Attribute) -> Result<(TokenStream, Vec<Meta>)> {
    let tokens = match attribute.meta.clone() {
        Meta::List(list) => list.tokens,
        _ => {
            return Err(Error::new_spanned(
                attribute,
                "malformed `cfg_attr` on VVM test",
            ));
        }
    };

    let mut condition = TokenStream::new();
    let mut nested = TokenStream::new();
    let mut found_separator = false;

    for token in tokens {
        let is_separator =
            matches!(&token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',');

        if !found_separator && is_separator {
            found_separator = true;
            continue;
        }

        if found_separator {
            nested.extend(std::iter::once(token));
        } else {
            condition.extend(std::iter::once(token));
        }
    }

    if !found_separator {
        return Err(Error::new_spanned(
            attribute,
            "malformed `cfg_attr` on VVM test",
        ));
    }

    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = parser.parse2(nested)?.into_iter().collect();

    Ok((condition, metas))
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{ItemFn, parse_quote};

    use super::{Input, TestArgument};

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
        assert!(matches!(input.argument, TestArgument::Config));
        Ok(())
    }

    #[test]
    fn accepts_mutable_test_context() -> Result<(), Box<dyn std::error::Error>> {
        let item: ItemFn = parse_quote! {
            /// Covered smoke test.
            fn covered(context: &mut vvm::TestContext) -> ResultType {
                run(context.config())
            }
        };

        let input = Input::parse(quote!(trace), item)?;

        assert!(matches!(input.argument, TestArgument::Context));
        Ok(())
    }

    #[test]
    fn rejects_immutable_test_context() {
        let item: ItemFn = parse_quote! {
            /// Invalid test.
            fn invalid(context: &vvm::TestContext) -> ResultType { run(context.config()) }
        };

        assert!(Input::parse(TokenStream::new(), item).is_err());
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

    #[test]
    fn rejects_should_panic() {
        let item: ItemFn = parse_quote! {
            /// Smoke test.
            #[should_panic]
            fn smoke() -> ResultType {
                run()
            }
        };

        assert!(Input::parse(TokenStream::new(), item).is_err());
    }

    #[test]
    fn rejects_manual_test_attribute() {
        let item: ItemFn = parse_quote! {
            /// Smoke test.
            #[test]
            fn smoke() -> ResultType {
                run()
            }
        };

        assert!(Input::parse(TokenStream::new(), item).is_err());
    }

    #[test]
    fn parses_coverage_context_and_partitions_attributes() -> Result<(), Box<dyn std::error::Error>>
    {
        let item: ItemFn = parse_quote! {
            /// Covered regression.
            #[cfg(feature = "native")]
            #[cfg_attr(feature = "native", cfg(unix), inline)]
            fn covered(context: &mut vvm::TestContext) -> ResultType {
                run(context.config())
            }
        };

        let input = Input::parse(quote!(coverage), item)?;

        assert!(input.coverage);
        assert!(matches!(input.argument, TestArgument::Context));
        assert_eq!(input.implementation.sig.ident, "__vvm_test_impl_covered");
        assert_eq!(input.wrapper_attributes.len(), 3);
        assert_eq!(input.implementation.attrs.len(), 2);
        assert_eq!(input.helper_attributes.len(), 2);
        assert_eq!(
            input
                .implementation
                .attrs
                .get(1)
                .and_then(|attribute| attribute.path().get_ident())
                .map(ToString::to_string)
                .as_deref(),
            Some("cfg_attr")
        );
        assert_eq!(
            input
                .helper_attributes
                .get(1)
                .and_then(|attribute| attribute.path().get_ident())
                .map(ToString::to_string)
                .as_deref(),
            Some("cfg_attr")
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_function_forms_with_specific_diagnostics() {
        let cases: [(ItemFn, TokenStream, &str); 7] = [
            (
                parse_quote!(
                    #[doc = "Test."]
                    const fn invalid() -> ResultType {
                        run()
                    }
                ),
                TokenStream::new(),
                "VVM tests must not be `const` functions",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    async fn invalid() -> ResultType {
                        run()
                    }
                ),
                TokenStream::new(),
                "VVM tests must be synchronous functions",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    unsafe fn invalid() -> ResultType {
                        run()
                    }
                ),
                TokenStream::new(),
                "VVM tests must not be `unsafe` functions",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    fn invalid<T>() -> ResultType {
                        run()
                    }
                ),
                TokenStream::new(),
                "VVM tests must not be generic",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    fn invalid() {
                        run()
                    }
                ),
                TokenStream::new(),
                "VVM tests must return a value convertible into `TestOutcome`",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    fn invalid(
                        first: &vvm::TestRunConfig,
                        second: &vvm::TestRunConfig,
                    ) -> ResultType {
                        run(first)
                    }
                ),
                TokenStream::new(),
                "VVM tests accept at most one `&TestRunConfig` or `&mut TestContext` argument",
            ),
            (
                parse_quote!(
                    #[doc = "Test."]
                    fn invalid(config: &mut vvm::TestRunConfig) -> ResultType {
                        run(config)
                    }
                ),
                TokenStream::new(),
                "VVM test configuration must be an immutable `&TestRunConfig` reference",
            ),
        ];

        for (item, attributes, expected) in cases {
            let result = Input::parse(attributes, item);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn rejects_invalid_metadata_and_coverage_without_context() {
        let invalid_name: ItemFn = parse_quote! {
            /// Test.
            fn invalid() -> ResultType { run() }
        };
        let missing_description: ItemFn = parse_quote! {
            fn undocumented() -> ResultType { run() }
        };
        let coverage_without_context: ItemFn = parse_quote! {
            /// Test.
            fn covered(config: &vvm::TestRunConfig) -> ResultType { run(config) }
        };

        let cases = [
            (
                Input::parse(quote!(name = "Invalid"), invalid_name),
                "invalid VVM test name `Invalid`; names must begin with an ASCII lowercase letter \
                 and contain only lowercase letters, digits, `-`, `_`, or `.`",
            ),
            (
                Input::parse(TokenStream::new(), missing_description),
                "VVM tests require a rustdoc description or `description = \"...\"`",
            ),
            (
                Input::parse(quote!(coverage), coverage_without_context),
                "tests declaring `coverage` must accept `&mut TestContext`",
            ),
        ];

        for (result, expected) in cases {
            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }
}
