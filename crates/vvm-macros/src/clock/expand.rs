use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;

use super::input::Input;

/// Expands a parsed `Clock` input.
pub(super) fn expand(input: Input) -> TokenStream {
    let Input {
        ident,
        mut generics,
        dut,
        setter,
        inactive,
        active,
    } = input;

    generics.make_where_clause().predicates.push(parse_quote!(
        #dut: ::vvm::Dut
    ));

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let setter_span = setter.span();

    let drive_inactive = quote_spanned! {setter_span=>
        __vvm_dut.#setter(#inactive)
    };

    let drive_active = quote_spanned! {setter_span=>
        __vvm_dut.#setter(#active)
    };

    quote! {
        impl #impl_generics
            ::vvm::Clock<#dut>
            for #ident #type_generics
            #where_clause
        {
            fn drive_inactive(
                &mut self,
                __vvm_dut: &mut #dut,
            ) -> ::core::result::Result<
                (),
                <#dut as ::vvm::Dut>::Error,
            > {
                #drive_inactive
            }

            fn drive_active(
                &mut self,
                __vvm_dut: &mut #dut,
            ) -> ::core::result::Result<
                (),
                <#dut as ::vvm::Dut>::Error,
            > {
                #drive_active
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::expand;
    use crate::clock::input::Input;

    #[test]
    fn expands_rising_edge_clock() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse_quote! {
            #[derive(Clock)]
            #[vvm(
                dut = crate::Counter,
                clock = "clk"
            )]
            struct CounterClock;
        };

        let tokens = expand(Input::parse(input)?).to_string();

        assert!(tokens.contains("vvm :: Clock"));
        assert!(tokens.contains("Clock"));
        assert!(tokens.contains("set_clk"));
        assert!(tokens.contains("drive_inactive"));
        assert!(tokens.contains("drive_active"));

        // Default rising edge:
        // inactive = false, active = true.
        assert!(tokens.contains("set_clk (false)"));
        assert!(tokens.contains("set_clk (true)"));

        Ok(())
    }

    #[test]
    fn expands_falling_edge_clock() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse_quote! {
            #[derive(Clock)]
            #[vvm(
                dut = crate::Counter,
                clock = "clk",
                edge = "falling"
            )]
            struct CounterClock;
        };

        let tokens = expand(Input::parse(input)?).to_string();

        let inactive = tokens.find("set_clk (true)");
        let active = tokens.rfind("set_clk (false)");

        assert!(inactive.is_some());
        assert!(active.is_some());
        assert!(inactive < active);

        Ok(())
    }
}
