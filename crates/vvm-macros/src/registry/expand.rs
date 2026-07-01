use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, PathArguments, Result};

use crate::registry::input::{Entry, Input};
use crate::test::names::descriptor_ident;

/// Expands one explicit ordered test registry.
///
/// # Errors
///
/// Returns an error when an entry is not a plain function path.
pub(super) fn expand(input: Input) -> Result<TokenStream> {
    let Input {
        attributes,
        visibility,
        ident,
        entries,
    } = input;

    let entries = entries
        .into_iter()
        .map(expand_entry)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #(#attributes)*
        #visibility static #ident: &[::vvm::TestDescriptor] = &[
            #(#entries,)*
        ];
    })
}

/// Replaces a test function's final path segment with its hidden descriptor.
fn expand_entry(entry: Entry) -> Result<TokenStream> {
    let Entry {
        attributes,
        mut path,
    } = entry;

    let Some(segment) = path.segments.last_mut() else {
        return Err(Error::new_spanned(
            path,
            "test registry entries must be function paths",
        ));
    };

    if !matches!(&segment.arguments, PathArguments::None) {
        return Err(Error::new_spanned(
            &segment.arguments,
            "test registry entries must not contain generic arguments",
        ));
    }

    segment.ident = descriptor_ident(&segment.ident);

    Ok(quote! {
        #(#attributes)*
        #path
    })
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::expand;
    use crate::registry::input::Input;

    #[test]
    fn preserves_order_and_qualified_paths() -> Result<(), Box<dyn std::error::Error>> {
        let input = parse2::<Input>(quote! {
            pub static TESTS = [
                smoke::counter_reset,
                random::counter_regression,
            ];
        })?;

        let tokens = expand(input)?.to_string();

        let first = tokens.find("smoke :: __vvm_test_descriptor_counter_reset");
        let second = tokens.find("random :: __vvm_test_descriptor_counter_regression");

        assert!(matches!(
            (first, second),
            (Some(first), Some(second)) if first < second
        ));
        Ok(())
    }
}
