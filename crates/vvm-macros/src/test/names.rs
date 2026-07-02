use quote::format_ident;
use syn::Ident;
use syn::ext::IdentExt;

/// Creates a hidden adapter function identifier.
pub fn adapter_ident(function: &Ident) -> Ident {
    format_ident!("__vvm_test_adapter_{}", function.unraw())
}

/// Creates a hidden typed implementation function identifier.
pub fn implementation_ident(function: &Ident) -> Ident {
    format_ident!("__vvm_test_impl_{}", function.unraw())
}

/// Creates a hidden descriptor identifier for a test function.
pub fn descriptor_ident(function: &Ident) -> Ident {
    format_ident!("__vvm_test_descriptor_{}", &function.unraw())
}
