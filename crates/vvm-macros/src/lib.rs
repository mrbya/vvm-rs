//! Procedural macros for VVM.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Shared VVM helper-attribute parsing.
mod attrs;
/// Drive trait derives.
mod drive;
/// Common derive-input diagnostics.
mod error;
/// Sample traits derives
mod sample;

/// Derives [`vvm_core::Drive`] for a named-field stimulus structure.
///
/// The target DUT is selected with `#[vvm(dut = path)]`. Fields marked with
/// `#[vvm(port)]` drive matching setters, while
/// `#[vvm(port = "name")]` selects an explicit DUT port.
#[proc_macro_derive(Drive, attributes(vvm))]
pub fn derive_drive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    drive::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives [`vvm_core::Sample`] for a named-field observation structure.
///
/// The target DUT is selected with `#[vvm(dut = path)]`. Every field must be
/// mapped with `#[vvm(port)]` or `#[vvm(port = "name")]`.
#[proc_macro_derive(Sample, attributes(vvm))]
pub fn derive_sample(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    sample::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
