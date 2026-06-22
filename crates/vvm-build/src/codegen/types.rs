use crate::metadata::BitWidth;

/// Public scalar type used by generated adapter method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    /// One-bit Boolean signal.
    Bool,

    /// Unsigned signal represented by 8 bits.
    U8,

    /// Unsigned signal represented by 16 bits.
    U16,

    /// Unsigned signal represented by 32 bits.
    U32,

    /// Unsigned signal represented by 64 bits.
    U64,
}

impl SignalType {
    /// Selects the public type for a normalized packed width.
    pub const fn from_width(width: BitWidth) -> Self {
        match width.get() {
            1 => Self::Bool,
            2..=8 => Self::U8,
            9..=16 => Self::U16,
            17..=32 => Self::U32,
            _ => Self::U64,
        }
    }

    /// Returns the generated public C++ type.
    pub const fn cpp_type(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "std::uint8_t",
            Self::U16 => "std::uint16_t",
            Self::U32 => "std::uint32_t",
            Self::U64 => "std::uint64_t",
        }
    }

    /// Returns the generated Rust type used by CXX.
    pub const fn rust_type(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
        }
    }

    /// Returns a C++ expression representing the packed-width mask.
    ///
    /// No mask is returned when the HDL width fills the complete public type.
    pub fn mask_literal(self, width: BitWidth) -> Option<String> {
        if width.get() == self.storage_width() {
            return None;
        }

        let mask = 1_u64.checked_shl(width.get())?.checked_sub(1)?;

        Some(format!("static_cast<{}>(0x{mask:X}ULL)", self.cpp_type()))
    }

    /// Returns the number of bits represented by the public type.
    const fn storage_width(self) -> u32 {
        match self {
            Self::Bool => 1,
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 => 32,
            Self::U64 => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::num::NonZeroU32;

    use super::SignalType;
    use crate::metadata::BitWidth;

    fn width(value: u32) -> Result<BitWidth, io::Error> {
        NonZeroU32::new(value).map(BitWidth::new).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "test width must be non-zero")
        })
    }

    #[test]
    fn selects_public_types_at_boundaries() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (1, SignalType::Bool),
            (2, SignalType::U8),
            (8, SignalType::U8),
            (9, SignalType::U16),
            (16, SignalType::U16),
            (17, SignalType::U32),
            (32, SignalType::U32),
            (33, SignalType::U64),
            (64, SignalType::U64),
        ];

        for (bits, expected) in cases {
            assert_eq!(SignalType::from_width(width(bits)?), expected);
        }

        Ok(())
    }

    #[test]
    fn select_rust_types() {
        let cases = [
            (SignalType::Bool, "bool"),
            (SignalType::U8, "u8"),
            (SignalType::U16, "u16"),
            (SignalType::U32, "u32"),
            (SignalType::U64, "u64"),
        ];

        for (signal, expected) in cases {
            assert_eq!(SignalType::rust_type(signal), expected);
        }
    }

    #[test]
    fn creates_mask_for_narrow_public_type() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            SignalType::U8.mask_literal(width(3)?),
            Some("static_cast<std::uint8_t>(0x7ULL)".to_owned())
        );

        assert_eq!(
            SignalType::U16.mask_literal(width(9)?),
            Some("static_cast<std::uint16_t>(0x1FFULL)".to_owned())
        );

        Ok(())
    }

    #[test]
    fn omits_mask_for_full_public_type() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(SignalType::U8.mask_literal(width(8)?), None);

        assert_eq!(SignalType::U64.mask_literal(width(64)?), None);

        Ok(())
    }
}
