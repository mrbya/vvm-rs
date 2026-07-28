use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::coverage::input::{FieldRole, Input};

/// Expands generated coverage wiring.
pub(super) fn expand(input: Input) -> TokenStream {
    let Input {
        ident: model_ident,
        definition,
        revision,
        stimulus,
        observation,
        fields,
    } = input;
    let constructors = fields.iter().map(|field| match *field {
        FieldRole::Coverpoint { ref ident, ref build, .. } => { let name = ident.to_string(); quote!(let #ident = #build(#name).map_err(|source| ::vvm::__private::CoverageDefinitionError::Coverpoint { item: #name, source })?;) }
        FieldRole::Cross { ref ident, ref left, ref right, ref build } => { let name = ident.to_string(); build.as_ref().map_or_else(|| quote!(let #ident = ::vvm::__private::Cross2::builder(#name, &#left, &#right).build().map_err(|source| ::vvm::__private::CoverageDefinitionError::Cross { item: #name, source })?;), |builder| quote!(let #ident = #builder(#name, &#left, &#right).map_err(|source| ::vvm::__private::CoverageDefinitionError::Cross { item: #name, source })?;)) }
    });
    let idents = fields.iter().map(|field| match *field {
        FieldRole::Coverpoint { ref ident, .. } | FieldRole::Cross { ref ident, .. } => ident,
    });
    let visits = fields.iter().map(|field| match *field {
        FieldRole::Coverpoint { ref ident, .. } => {
            quote!(visitor.visit(::vvm::__private::CoverageItemRef::coverpoint(&self.#ident));)
        }
        FieldRole::Cross { ref ident, .. } => {
            quote!(visitor.visit(::vvm::__private::CoverageItemRef::cross2(&self.#ident));)
        }
    });
    let samples = fields.iter().filter_map(|field| match *field { FieldRole::Coverpoint { ref ident, ref sample, .. } => Some((ident, sample)), FieldRole::Cross { .. } => None }).map(|(field, sample)| { let value = format_ident!("__vvm_{}_value", field); let result = format_ident!("__vvm_{}_sample", field); let name = field.to_string(); quote!(let #value = #sample(cycle); let #result = self.#field.sample(&#value).map_err(|source| ::vvm::__private::CoverageRuntimeError::Coverpoint { item: #name, cycle: cycle.cycle(), time: cycle.time(), source })?;) });
    let crosses = fields.iter().filter_map(|field| match *field { FieldRole::Cross { ref ident, ref left, ref right, .. } => Some((ident, left, right)), FieldRole::Coverpoint { .. } => None }).map(|(field, left, right)| { let left_sample = format_ident!("__vvm_{}_sample", left); let right_sample = format_ident!("__vvm_{}_sample", right); let name = field.to_string(); quote!(self.#field.sample(&#left_sample, &#right_sample).map_err(|source| ::vvm::__private::CoverageRuntimeError::Cross { item: #name, cycle: cycle.cycle(), time: cycle.time(), source })?;) });
    quote! {
        impl #model_ident {
            /// Constructs one validated coverage instance.
            pub fn new(instance_path: impl ::core::convert::Into<::std::string::String>) -> ::core::result::Result<::vvm::__private::CoverageInstance<Self>, ::vvm::__private::CoverageDefinitionError> {
                #(#constructors)*
                let model = Self { #(#idents),* };
                ::vvm::__private::CoverageInstance::__vvm_new(instance_path, model)
            }
        }
        impl ::vvm::__private::CoverageSpec for #model_ident {
            const DEFINITION_NAME: &'static str = #definition;
            const DEFINITION_REVISION: u64 = #revision;
            fn visit_coverage_items(&self, visitor: &mut dyn ::vvm::__private::CoverageGroupVisitor) { #(#visits)* }
        }
        impl ::vvm::__private::CoverageSampleSpec<#stimulus, #observation> for #model_ident {
            fn sample_coverage_items(&mut self, cycle: ::vvm::__private::ObservedCycle<'_, #stimulus, #observation>) -> ::core::result::Result<(), ::vvm::__private::CoverageRuntimeError> { #(#samples)* #(#crosses)* ::core::result::Result::Ok(()) }
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{DeriveInput, parse2};

    use super::expand;
    use crate::coverage::input::Input;

    #[test]
    fn generates_constructor_visitation_and_sampling() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
            struct Model {
                #[vvm(coverpoint(build = build_left, sample = sample_left))]
                left: Coverpoint<u8>,
                #[vvm(coverpoint(build = build_right, sample = sample_right))]
                right: Coverpoint<bool>,
                #[vvm(cross(left = left, right = right))]
                cross: Cross2,
            }
        })?;
        let tokens = expand(Input::parse(input)?).to_string();

        for expected in [
            "CoverageSpec",
            "DEFINITION_NAME",
            "CoverageInstance",
            "build_left",
            "CoverageDefinitionError",
            "CoverageItemRef",
            "sample_left",
            "CoverageRuntimeError",
            "__vvm_left_sample",
        ] {
            assert!(
                tokens.contains(expected),
                "missing generated token {expected}"
            );
        }

        Ok(())
    }
}
