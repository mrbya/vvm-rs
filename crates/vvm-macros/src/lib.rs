//! Procedural macros for VVM.

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

/// Shared VVM helper-attribute parsing.
mod attrs;
/// Clock trait derives.
mod clock;
/// Coverage model derives.
mod coverage;
/// Common compile-time derive-input diagnostics.
mod diagnostic;
/// Drive trait derives.
mod drive;
/// Sample traits derives
mod sample;
/// VVM test attribute expansion.
mod test;

/// Derives `vvm::Drive` for a named-field stimulus structure.
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

/// Derives `vvm::Sample` for a named-field observation structure.
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

/// Derives `vvm::Clock` for a unit clock-driver type.
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

/// Derives typed functional-coverage construction, visitation, and sampling.
#[proc_macro_derive(Coverage, attributes(vvm))]
pub fn derive_coverage(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    coverage::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Registers a typed function as a VVM test.
///
/// The attribute preserves the original function name as the visible Rust
/// `#[test]` wrapper and generates hidden implementation and adapter helpers.
#[proc_macro_attribute]
pub fn test(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    test::expand(attributes.into(), item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
