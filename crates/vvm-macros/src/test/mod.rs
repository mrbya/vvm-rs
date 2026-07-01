use proc_macro2::TokenStream;
use syn::{ItemFn, Result};

/// Parses `#[vvm::test(...)]` arguments.
mod attrs;
/// Expands validated VVM test functions.
mod expand;
/// Parses and validates VVM test functions.
mod input;
/// Creates stable hidden item identifiers.
pub mod names;

/// Parses and expands one `#[vvm::test]` function.
///
/// # Errors
///
/// Returns an error for invalid attributes or unsupported function forms.
pub fn expand(attributes: TokenStream, item: ItemFn) -> Result<TokenStream> {
    Ok(expand::expand(input::Input::parse(attributes, item)?))
}
