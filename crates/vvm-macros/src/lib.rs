//! Procedural macros for VVM.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Shared VVM helper-attribute parsing.
mod attrs;
/// Clock trait derives.
mod clock;
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

/// Derives [`vvm_core::Clock`] for a unit clock-driver type.
///
/// The target DUT and clock port are configured with:
///
/// ```no_run
/// #[vvm(dut = path, clock = "port")]
/// ```
///
/// Rising-edge behavior is the default. A falling-edge clock may be selected
/// with `edge = "falling"`.
#[proc_macro_derive(Clock, attributes(vvm))]
pub fn derive_clock(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    clock::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
