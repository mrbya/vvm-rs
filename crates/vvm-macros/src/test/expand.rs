use proc_macro2::TokenStream;
use quote::quote;

use crate::test::attrs::ReplayAttribute;
use crate::test::input::Input;
use crate::test::names::{adapter_ident, descriptor_ident};

/// Expands one validated VVM test function.
pub(super) fn expand(input: Input) -> TokenStream {
    let Input {
        item,
        name,
        description,
        trace,
        cycles,
        replay,
        accepts_config,
        cfg_attributes,
    } = input;

    let function = item.sig.ident.clone();
    let adapter = adapter_ident(&function);
    let descriptor = descriptor_ident(&function);

    let call = if accepts_config {
        quote!(#function(__vvm_config))
    } else {
        quote!(#function())
    };

    let adapter_body = match &replay {
        ReplayAttribute::Disabled => quote! {
            ::vvm::IntoTestOutcome::into_test_outcome(#call)
        },
        ReplayAttribute::Enabled {
            default: Some(default),
        } => quote! {
            let __vvm_result = #call;
            let __vvm_replay = __vvm_config
                .replay_token()
                .unwrap_or(#default);

            ::vvm::IntoTestOutcome::into_test_outcome_with_replay(
                __vvm_result,
                __vvm_replay,
            )
        },
        ReplayAttribute::Enabled { default: None } => quote! {
            let __vvm_result = #call;

            match __vvm_config.replay_token() {
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
        },
    };

    let mut capabilities = quote!(::vvm::TestCapabilities::new());

    if trace {
        capabilities = quote!((#capabilities).with_trace());
    }

    if cycles {
        capabilities = quote!((#capabilities).with_cycles());
    }

    capabilities = match &replay {
        ReplayAttribute::Disabled => capabilities,
        ReplayAttribute::Enabled { default: None } => {
            quote!((#capabilities).with_replay())
        }
        ReplayAttribute::Enabled {
            default: Some(default),
        } => quote!((#capabilities).with_default_replay(#default)),
    };

    quote! {
        #item

        #(#cfg_attributes)*
        #[doc(hidden)]
        #[allow(dead_code)]
        fn #adapter(
            __vvm_config: &::vvm::TestRunConfig,
        ) -> ::vvm::TestOutcome {
            #adapter_body
        }

        #(#cfg_attributes)*
        #[doc(hidden)]
        #[allow(dead_code, non_upper_case_globals)]
        pub(crate) const #descriptor: ::vvm::TestDescriptor =
            ::vvm::TestDescriptor::new(
                #name,
                #description,
                #adapter,
                #capabilities,
            );
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{ItemFn, parse_quote};

    use super::expand;
    use crate::test::input::Input;

    #[test]
    fn expands_adapter_descriptor_and_capabilities() -> Result<(), Box<dyn std::error::Error>> {
        let item: ItemFn = parse_quote! {
            /// Randomized counter regression.
            fn counter_random(config: &vvm::TestRunConfig) -> ResultType {
                run(config)
            }
        };

        let tokens = expand(Input::parse(
            quote!(trace, cycles, replay(default = DEFAULT_REPLAY)),
            item,
        )?)
        .to_string();

        assert!(tokens.contains("__vvm_test_adapter_counter_random"));
        assert!(tokens.contains("__vvm_test_descriptor_counter_random"));
        assert!(tokens.contains("with_trace"));
        assert!(tokens.contains("with_cycles"));
        assert!(tokens.contains("with_default_replay"));
        assert!(tokens.contains("into_test_outcome_with_replay"));
        Ok(())
    }
}
