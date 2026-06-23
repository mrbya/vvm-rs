use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;

use crate::drive::input::Input;

/// Expands a parsed `Drive` input.
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
        #dut: ::vvm_core::Dut
    ));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let drive_statements = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let setter = &field.setter;
        let span = setter.span();

        quote_spanned! {span=>
            __vvm_dut.#setter(self.#field_ident)?;
        }
    });

    quote! {
        impl #impl_generics
            ::vvm_core::Drive<#dut>
            for #ident #type_generics
            #where_clause
        {
            fn drive(
                &self,
                __vvm_dut: &mut #dut,
            ) -> ::core::result::Result<
                (),
                <#dut as ::vvm_core::Dut>::Error,
            > {
                #(#drive_statements)*

                ::core::result::Result::Ok(())
            }
        }
    }
}
