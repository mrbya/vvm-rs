use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::test::input::{Input, TestArgument};
use crate::test::names::{adapter_ident, descriptor_ident};

/// Expands one validated VVM test function.
pub(super) fn expand(input: Input) -> TokenStream {
    let Input {
        implementation,
        function,
        wrapper_attributes,
        helper_attributes,
        name,
        description,
        trace,
        cycles,
        replay,
        coverage,
        argument,
    } = input;

    let adapter = adapter_ident(&function);
    let descriptor = descriptor_ident(&function);
    let implementation_call = implementation_call(&implementation, argument);

    let adapter_body = if replay.enabled() {
        quote! {
            let __vvm_result = #implementation_call;

            match __vvm_context.config().replay_token() {
                ::core::option::Option::Some(__vvm_replay) => {
                    ::vvm::IntoTestOutcome::into_test_outcome_with_replay(
                        __vvm_result,
                        __vvm_replay,
                    )
                }
                ::core::option::Option::None => {
                    ::vvm::IntoTestOutcome::into_test_outcome(__vvm_result)
                }
            }
        }
    } else {
        quote! {
            ::vvm::IntoTestOutcome::into_test_outcome(#implementation_call)
        }
    };

    let mut capabilities = quote!(::vvm::TestCapabilities::new());

    if trace {
        capabilities = quote!((#capabilities).with_trace());
    }

    if cycles {
        capabilities = quote!((#capabilities).with_cycles());
    }

    if let Some(default) = replay.default_expr() {
        capabilities = quote!((#capabilities).with_default_replay(#default));
    } else if replay.enabled() {
        capabilities = quote!((#capabilities).with_replay());
    }
    if coverage {
        capabilities = quote!((#capabilities).with_coverage());
    }

    quote! {
        #implementation

        #(#helper_attributes)*
        #[doc(hidden)]
        #[allow(dead_code)]
        fn #adapter(__vvm_context: &mut ::vvm::TestContext) -> ::vvm::TestOutcome {
            #adapter_body
        }

        #(#helper_attributes)*
        #[doc(hidden)]
        #[allow(dead_code, non_upper_case_globals)]
        const #descriptor: ::vvm::TestDescriptor = ::vvm::TestDescriptor::new_with_context(
            #name,
            #description,
            #adapter,
            #capabilities,
        );

        #(#wrapper_attributes)*
        #[test]
        fn #function() -> ::core::result::Result<(), ::vvm::__private::TestFailure> {
            ::vvm::__private::run_test(&#descriptor)
        }
    }
}

/// Builds the call from the generated adapter into the hidden implementation.
fn implementation_call(implementation: &ItemFn, argument: TestArgument) -> TokenStream {
    let function = &implementation.sig.ident;

    match argument {
        TestArgument::None => quote!(#function()),
        TestArgument::Config => quote!(#function(__vvm_context.config())),
        TestArgument::Context => quote!(#function(__vvm_context)),
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{Attribute, File, Item, ItemConst, ItemFn, parse_quote, parse2};

    use super::expand;
    use crate::test::input::Input;

    #[test]
    fn expands_standard_test_wrapper_and_hidden_helpers() -> Result<(), Box<dyn std::error::Error>>
    {
        let item: ItemFn = parse_quote! {
            /// Randomized counter regression.
            #[ignore = "example"]
            fn counter_random(config: &vvm::TestRunConfig) -> ResultType {
                run(config)
            }
        };

        let file: File = parse2(expand(Input::parse(
            quote!(trace, cycles, replay(default = DEFAULT_REPLAY)),
            item,
        )?))?;

        let items = &file.items;

        if items.len() != 4 {
            return Err("expected four generated items".into());
        }

        let implementation = expect_fn(items.first(), "expected hidden implementation function")?;
        let adapter = expect_fn(items.get(1), "expected adapter function")?;
        let descriptor = expect_const(items.get(2), "expected descriptor constant")?;
        let wrapper = expect_fn(items.get(3), "expected visible test wrapper")?;

        assert_eq!(implementation.sig.ident, "__vvm_test_impl_counter_random");
        assert_eq!(adapter.sig.ident, "__vvm_test_adapter_counter_random");
        assert_eq!(descriptor.ident, "__vvm_test_descriptor_counter_random");
        assert_eq!(wrapper.sig.ident, "counter_random");
        assert!(has_attribute(&wrapper.attrs, "test"));
        assert!(has_attribute(&wrapper.attrs, "ignore"));
        assert!(has_attribute(&wrapper.attrs, "doc"));
        assert!(descriptor_contains(descriptor, "with_trace"));
        assert!(descriptor_contains(descriptor, "with_cycles"));
        assert!(descriptor_contains(descriptor, "with_default_replay"));
        assert!(function_contains(adapter, "__vvm_test_impl_counter_random"));
        Ok(())
    }

    #[test]
    fn propagates_cfg_to_hidden_helpers() -> Result<(), Box<dyn std::error::Error>> {
        let source: ItemFn = parse_quote! {
            /// Smoke test.
            #[cfg(feature = "sim")]
            fn smoke() -> ResultType {
                run()
            }
        };

        let file: File = parse2(expand(Input::parse(TokenStream::new(), source)?))?;

        for generated_item in &file.items {
            match *generated_item {
                Item::Fn(ref function) => assert!(has_attribute(&function.attrs, "cfg")),
                Item::Const(ref constant) => assert!(has_attribute(&constant.attrs, "cfg")),
                _ => return Err("unexpected generated item".into()),
            }
        }

        Ok(())
    }

    fn has_attribute(attributes: &[Attribute], ident: &str) -> bool {
        attributes
            .iter()
            .any(|attribute| attribute.path().is_ident(ident))
    }

    fn function_contains(function: &ItemFn, snippet: &str) -> bool {
        quote!(#function).to_string().contains(snippet)
    }

    fn descriptor_contains(descriptor: &ItemConst, snippet: &str) -> bool {
        quote!(#descriptor).to_string().contains(snippet)
    }

    fn expect_fn<'a>(
        item: Option<&'a Item>,
        message: &'static str,
    ) -> Result<&'a ItemFn, Box<dyn std::error::Error>> {
        let Some(item) = item else {
            return Err(message.into());
        };

        match *item {
            Item::Fn(ref function) => Ok(function),
            _ => Err(message.into()),
        }
    }

    fn expect_const<'a>(
        item: Option<&'a Item>,
        message: &'static str,
    ) -> Result<&'a ItemConst, Box<dyn std::error::Error>> {
        let Some(item) = item else {
            return Err(message.into());
        };

        match *item {
            Item::Const(ref constant) => Ok(constant),
            _ => Err(message.into()),
        }
    }
}
