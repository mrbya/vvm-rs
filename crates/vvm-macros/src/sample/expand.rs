use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;

use crate::sample::input::Input;

/// Expands a parsed `Sample` input.
///
/// # Errors
///
/// Returns an error if generated generic predicates cannot be constructed.
pub(super) fn expand(input: Input) -> TokenStream {
    let Input {
        ident,
        mut generics,
        dut,
        fields,
    } = input;

    generics.make_where_clause().predicates.push(parse_quote!(
        #dut: ::vvm::__private::Dut
    ));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let field_initializers = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let getter = &field.getter;
        let span = getter.span();

        quote_spanned! {span=>
            #field_ident: __vvm_dut.#getter()?,
        }
    });

    quote! {
        impl #impl_generics
            ::vvm::__private::Sample<#dut>
            for #ident #type_generics
            #where_clause
        {
            fn sample(
                __vvm_dut: &#dut,
            ) -> ::core::result::Result<
                Self,
                <#dut as ::vvm::__private::Dut>::Error,
            > {
                ::core::result::Result::Ok(Self {
                    #(#field_initializers)*
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::expand;
    use crate::sample::input::Input;

    #[test]
    fn expands_mapped_setters() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse_quote! {
            #[derive(Sample)]
            #[vvm(dut = crate::Counter)]
            struct Observation {
                #[vvm(port)]
                data_out: bool,

                #[vvm(port)]
                en_out: bool,
            }
        };

        let tokens = expand(Input::parse(input)?).to_string();

        assert!(tokens.contains("data_out"));
        assert!(tokens.contains("en_out"));
        assert!(tokens.contains("vvm :: __private :: Sample"));
        assert!(tokens.contains("vvm :: __private :: Dut"));
        assert!(!tokens.contains("vvm :: Sample <"));
        assert!(tokens.contains("Sample"));

        Ok(())
    }
}
