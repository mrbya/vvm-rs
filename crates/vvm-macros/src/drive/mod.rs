use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};

/// `Drive` implementation expansion.
mod expand;
/// Parsed `Drive` derive input.
mod input;

/// Parses and expands `Drive` trait derive.
///
/// # Errors
///
/// Returns an error for invalid derive input.
pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    Ok(expand::expand(input::Input::parse(input)?))
}
