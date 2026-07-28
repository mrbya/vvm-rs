use syn::{DeriveInput, Generics, Ident, Path, Result};

use crate::attrs::{parse_dut_path, parse_port_attribute};
use crate::diagnostic::named_fields;

/// Parsed `Drive` implementation input.
pub(super) struct Input {
    /// Stimulus type identifier.
    pub(super) ident: Ident,

    /// Stimulus type generics.
    pub(super) generics: Generics,

    /// DUT type path.
    pub(super) dut: Path,

    /// Mapped stimulus fields.
    pub(super) fields: Vec<Field>,
}

/// One field mapped to a generated DUT setter.
pub(super) struct Field {
    /// Stimulus field identifier.
    pub(super) ident: Ident,

    /// Generated DUT setter identifier.
    pub(super) setter: Ident,
}

impl Input {
    /// Parses a complete derive input.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported data shapes or invalid VVM metadata.
    pub(super) fn parse(input: DeriveInput) -> Result<Self> {
        let dut = parse_dut_path(&input.attrs, input.ident.span())?;

        let named_fields = named_fields(&input, "Drive")?;

        let mut fields = Vec::new();

        for field in &named_fields.named {
            let Some(ident) = field.ident.clone() else {
                continue;
            };

            let Some(port) = parse_port_attribute(&field.attrs)? else {
                continue;
            };

            let setter = port.method_ident(&ident, "set_")?;

            fields.push(Field { ident, setter });
        }

        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            dut,
            fields,
        })
    }
}
