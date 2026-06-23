use syn::{Data, DeriveInput, Error, Fields, FieldsNamed, Ident, Result};

/// Returns the named fields accepted by VVM derives.
///
/// # Errors
///
/// Returns an error when applied to an enum, union, tuple struct, or unit
/// struct.
pub fn named_fields<'a>(input: &'a DeriveInput, derive_name: &str) -> Result<&'a FieldsNamed> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields),
            Fields::Unnamed(_) | Fields::Unit => Err(Error::new_spanned(
                &input.ident,
                format!("`{derive_name}` can only be derived for a struct with named fields"),
            )),
        },

        Data::Enum(_) | Data::Union(_) => Err(Error::new_spanned(
            &input.ident,
            format!("`{derive_name}` can only be derived for a struct with named fields"),
        )),
    }
}

/// Creates the diagnostic for an unmapped `Sample` field.
pub fn unmapped_sample_field(field: &Ident) -> Error {
    Error::new_spanned(
        field,
        "`Sample` fields must be mapped with `#[vvm(port)]` or `#[vvm(port = \"name\")]`",
    )
}
