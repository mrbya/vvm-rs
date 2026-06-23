use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};

/// `Clock` implementation expansion.
mod expand;
/// Parses `Clock` derive input.
mod input;

/// Parses and expands `Clock` trait derive.
///
/// # Errors
///
/// Returns an error for invalid derive input or clock configuration.
pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    Ok(expand::expand(input::Input::parse(input)?))
}
