use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};

/// `Sample` implementation expansion.
mod expand;
/// Parses `Sample` input.
mod input;

/// Parses and expands `Sample`.
///
/// # Errors
///
/// Returns an error for invalid derive input.
pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    Ok(expand::expand(input::Input::parse(input)?))
}
