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

    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#dut ::vvm_core::Dut));

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
            ::vvm_core::Sample<#dut>
            for #ident #type_generics
            #where_clause
        {
            fn sample(
                __vvm_dut: &#dut,
            ) -> ::core::result::Result<
                Self,
                <#dut as ::vvm_core::Dut>::Error,
            > {
                ::core::result::Result::Ok(Self {
                    #(#field_initializers)*
                })
            }
        }
    }
}
