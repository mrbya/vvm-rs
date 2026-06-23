use syn::{DeriveInput, Generics, Ident, Path, Result};

use crate::{
    attrs::{parse_dut_path, parse_port_attribute},
    error::{named_fields, unmapped_sample_field},
};

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

        let named_fields = named_fields(&input, "sample")?;

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
