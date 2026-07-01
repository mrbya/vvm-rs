use proc_macro2::TokenStream;
use syn::{Result, parse2};

/// Expands explicit ordered VVM test registries.
mod expand;
/// Parses explicit ordered VVM test registries.
mod input;

/// Parses and expands one `test_registry!` invocation.
///
/// # Errors
///
/// Returns an error for malformed declarations or invalid test paths.
pub fn expand(tokens: TokenStream) -> Result<TokenStream> {
    expand::expand(parse2::<input::Input>(tokens)?)
}
