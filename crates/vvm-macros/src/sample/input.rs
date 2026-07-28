use syn::{DeriveInput, Generics, Ident, Path, Result};

use crate::attrs::{parse_dut_path, parse_port_attribute};
use crate::diagnostic::{named_fields, unmapped_sample_field};

/// Parsed `Sample` implementation input.
pub(super) struct Input {
    /// Observation type identifier.
    pub(super) ident: Ident,

    /// Observation type generics.
    pub(super) generics: Generics,

    /// DUT type path.
    pub(super) dut: Path,

    /// Mapped observation fields.
    pub(super) fields: Vec<Field>,
}

/// One observation field mapped to a DUT getter.
pub(super) struct Field {
    /// Observation field identifier.
    pub(super) ident: Ident,

    /// Generated DUT getter identifier.
    pub(super) getter: Ident,
}

impl Input {
    /// Parses a complete derive input.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported data shapes, unmapped fields, or
    /// invalid VVM metadata.
    pub(super) fn parse(input: DeriveInput) -> Result<Self> {
        let dut = parse_dut_path(&input.attrs, input.ident.span())?;

        let named_fields = named_fields(&input, "Sample")?;

        let mut fields = Vec::new();

        for field in &named_fields.named {
            let Some(ident) = field.ident.clone() else {
                continue;
            };

            let port =
                parse_port_attribute(&field.attrs)?.ok_or_else(|| unmapped_sample_field(&ident))?;

            let getter = port.method_ident(&ident, "")?;

            fields.push(Field { ident, getter });
        }

        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            dut,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{DeriveInput, parse2};

    use super::Input;

    #[test]
    fn parses_sample_fields_and_explicit_ports() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            #[vvm(dut = crate::Dut)]
            struct Observation<T> {
                #[vvm(port)]
                ready: bool,
                #[vvm(port = "data_out")]
                data: T,
            }
        })?;
        let parsed = Input::parse(input)?;
        let dut = &parsed.dut;

        assert_eq!(parsed.ident, "Observation");
        assert_eq!(quote!(#dut).to_string(), "crate :: Dut");
        assert_eq!(parsed.fields.len(), 2);
        assert!(matches!(
            parsed.fields.first(),
            Some(field) if field.ident == "ready" && field.getter == "ready"
        ));
        assert!(matches!(
            parsed.fields.get(1),
            Some(field) if field.getter == "data_out"
        ));
        Ok(())
    }

    #[test]
    fn rejects_invalid_sample_inputs_with_specific_diagnostics()
    -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                quote!(
                    struct Observation {
                        ready: bool,
                    }
                ),
                "missing `#[vvm(dut = path)]` attribute",
            ),
            (
                quote!(
                    #[vvm(dut = Dut)]
                    struct Observation {
                        ready: bool,
                    }
                ),
                "`Sample` fields must be mapped with `#[vvm(port)]` or `#[vvm(port = \"name\")]`",
            ),
            (
                quote!(
                    #[vvm(dut = Dut)]
                    struct Observation {
                        #[vvm(port = "bad-port")]
                        ready: bool,
                    }
                ),
                "port name `bad-port` does not produce a valid Rust method identifier",
            ),
            (
                quote!(
                    #[vvm(dut = Dut)]
                    struct Observation(bool);
                ),
                "`Sample` can only be derived for a struct with named fields",
            ),
            (
                quote!(
                    #[vvm(dut = Dut)]
                    enum Observation {
                        Ready,
                    }
                ),
                "`Sample` can only be derived for a struct with named fields",
            ),
        ];

        for (tokens, expected) in cases {
            let input: DeriveInput = parse2(tokens)?;
            let result = Input::parse(input);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }

        Ok(())
    }
}
