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
        #dut: ::vvm::Dut
    ));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let drive_statements = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let setter = &field.setter;
        let span = setter.span();

        quote_spanned! {span=>
            __vvm_dut.#setter(&self.#field_ident)?;
        }
    });

    quote! {
        #[allow(clippy::needless_pass_by_value)]
        #[allow(clippy::needless_borrows_for_generic_args)]
        impl #impl_generics
            ::vvm::Drive<#dut>
            for #ident #type_generics
            #where_clause
        {
            fn drive(
                &self,
                __vvm_dut: &mut #dut,
            ) -> ::core::result::Result<
                (),
                <#dut as ::vvm::Dut>::Error,
            > {
                #(#drive_statements)*

                ::core::result::Result::Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::expand;
    use crate::drive::input::Input;

    #[test]
    fn expands_mapped_setters() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse_quote! {
            #[derive(Drive)]
            #[vvm(dut = crate::Counter)]
            struct Stimulus {
                #[vvm(port)]
                enable: bool,

                #[vvm(port = "reset_n")]
                reset: bool,
            }
        };

        let tokens = expand(Input::parse(input)?).to_string();

        assert!(tokens.contains("set_enable"));
        assert!(tokens.contains("set_reset_n"));
        assert!(tokens.contains("vvm :: Drive"));

        assert!(tokens.contains("set_enable (& self . enable)",),);

        assert!(tokens.contains("set_reset_n (& self . reset)",),);

        Ok(())
    }
}
