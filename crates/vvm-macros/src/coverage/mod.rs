//! Coverage derive implementation.

/// Coverage token expansion.
mod expand;
/// Coverage input parsing.
mod input;

use syn::{DeriveInput, Result};

/// Derives typed functional-coverage wiring.
pub fn derive(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    Ok(expand::expand(input::Input::parse(input)?))
}
