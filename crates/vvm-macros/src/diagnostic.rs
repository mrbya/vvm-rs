use syn::{Data, DeriveInput, Error, Fields, FieldsNamed, Ident, Result};

/// Returns the named fields accepted by VVM derives.
///
/// # Errors
///
/// Returns an error when applied to an enum, union, tuple struct, or unit
/// struct.
pub fn named_fields<'a>(input: &'a DeriveInput, derive_name: &str) -> Result<&'a FieldsNamed> {
    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => Ok(fields),
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

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{DeriveInput, parse_quote, parse2};

    use super::{named_fields, unmapped_sample_field};

    #[test]
    fn returns_named_struct_fields() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            struct Observation {
                ready: bool,
                data: u8,
            }
        })?;

        let fields = named_fields(&input, "Sample")?;

        assert_eq!(fields.named.len(), 2);
        assert_eq!(
            fields
                .named
                .first()
                .and_then(|field| field.ident.as_ref())
                .map(ToString::to_string)
                .as_deref(),
            Some("ready")
        );
        Ok(())
    }

    #[test]
    fn rejects_non_named_shapes_with_derive_specific_diagnostic() {
        let cases: [DeriveInput; 3] = [
            parse_quote!(
                struct Tuple(u8);
            ),
            parse_quote!(
                enum Kind {
                    One,
                }
            ),
            parse_quote!(union Bits { raw: u8 }),
        ];

        for input in cases {
            let result = named_fields(&input, "Drive");

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some("`Drive` can only be derived for a struct with named fields")
            );
        }
    }

    #[test]
    fn reports_unmapped_sample_field() {
        let error = unmapped_sample_field(&parse_quote!(status));

        assert_eq!(
            error.to_string(),
            "`Sample` fields must be mapped with `#[vvm(port)]` or `#[vvm(port = \"name\")]`"
        );
    }
}
