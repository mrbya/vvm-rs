use crate::metadata::{BitWidth, Port};

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

    /// Signed signal represented by 8 bits.
    I8,

    /// Signed signal represented by 16 bits.
    I16,

    /// Signed signal represented by 32 bits,
    I32,

    /// Signed signal represented by 64 bits.
    I64,
}

impl SignalType {
    /// Selects the public scalar type for a normalized port.
    pub const fn from_port(port: &Port) -> Self {
        match (port.signed, port.width.get()) {
            (false, 1) => Self::Bool,
            (false, 2..=8) => Self::U8,
            (false, 9..=16) => Self::U16,
            (false, 17..=32) => Self::U32,
            (false, _) => Self::U64,

            (true, 1..=8) => Self::I8,
            (true, 9..=16) => Self::I16,
            (true, 17..=32) => Self::I32,
            (true, _) => Self::I64,
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
            Self::I8 => "std::int8_t",
            Self::I16 => "std::int16_t",
            Self::I32 => "std::int32_t",
            Self::I64 => "std::int64_t",
        }
    }

    /// Returns the corresponding unsigned C++ storage type.
    ///
    /// Signed signal conversion uses this type to manipulate the underlying
    /// two's-complement bit pattern without relying on signed bit operations.
    pub const fn unsigned_cpp_type(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 | Self::I8 => "std::uint8_t",
            Self::U16 | Self::I16 => "std::uint16_t",
            Self::U32 | Self::I32 => "std::uint32_t",
            Self::U64 | Self::I64 => "std::uint64_t",
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
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
        }
    }

    /// Returns a C++ expression representing the packed-width mask.
    ///
    /// The mask always uses an unsigned type, including for signed ports.
    /// No mask is returned when the HDL width fills the public storage type.
    pub fn mask_literal(self, width: BitWidth) -> Option<String> {
        if width.get() == self.storage_width() {
            return None;
        }

        let mask = 1_u64.checked_shl(width.get())?.checked_sub(1)?;

        Some(format!(
            "static_cast<{}>(0x{mask:X}ULL)",
            self.unsigned_cpp_type(),
        ))
    }

    /// Returns the number of bits represented by the public type.
    const fn storage_width(self) -> u32 {
        match self {
            Self::Bool => 1,
            Self::U8 | Self::I8 => 8,
            Self::U16 | Self::I16 => 16,
            Self::U32 | Self::I32 => 32,
            Self::U64 | Self::I64 => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::num::NonZeroU32;

    use super::SignalType;
    use crate::metadata::{BitWidth, Port, PortDirection};

    fn width(value: u32) -> Result<BitWidth, io::Error> {
        NonZeroU32::new(value).map(BitWidth::new).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "test width must be non-zero")
        })
    }

    fn signal_type(bits: u32, signed: bool) -> Result<SignalType, io::Error> {
        let port = Port {
            name: String::from("value"),
            direction: PortDirection::Input,
            width: width(bits)?,
            signed,
        };

        Ok(SignalType::from_port(&port))
    }

    #[test]
    fn selects_unsigned_public_types_at_boundaries() -> Result<(), Box<dyn std::error::Error>> {
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
            assert_eq!(signal_type(bits, false)?, expected);
        }

        Ok(())
    }

    #[test]
    fn selects_signed_public_types_at_boundaries() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (1, SignalType::I8),
            (2, SignalType::I8),
            (8, SignalType::I8),
            (9, SignalType::I16),
            (16, SignalType::I16),
            (17, SignalType::I32),
            (32, SignalType::I32),
            (33, SignalType::I64),
            (64, SignalType::I64),
        ];

        for (bits, expected) in cases {
            assert_eq!(signal_type(bits, true)?, expected);
        }

        Ok(())
    }

    #[test]
    fn selects_cpp_types() {
        let cases = [
            (SignalType::Bool, "bool"),
            (SignalType::U8, "std::uint8_t"),
            (SignalType::U16, "std::uint16_t"),
            (SignalType::U32, "std::uint32_t"),
            (SignalType::U64, "std::uint64_t"),
            (SignalType::I8, "std::int8_t"),
            (SignalType::I16, "std::int16_t"),
            (SignalType::I32, "std::int32_t"),
            (SignalType::I64, "std::int64_t"),
        ];

        for (signal, expected) in cases {
            assert_eq!(signal.cpp_type(), expected);
        }
    }

    #[test]
    fn selects_rust_types() {
        let cases = [
            (SignalType::Bool, "bool"),
            (SignalType::U8, "u8"),
            (SignalType::U16, "u16"),
            (SignalType::U32, "u32"),
            (SignalType::U64, "u64"),
            (SignalType::I8, "i8"),
            (SignalType::I16, "i16"),
            (SignalType::I32, "i32"),
            (SignalType::I64, "i64"),
        ];

        for (signal, expected) in cases {
            assert_eq!(signal.rust_type(), expected);
        }
    }

    #[test]
    fn creates_unsigned_mask_for_narrow_signed_signal() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            SignalType::I8.mask_literal(width(5)?),
            Some("static_cast<std::uint8_t>(0x1FULL)".to_owned(),),
        );

        assert_eq!(
            SignalType::I16.mask_literal(width(9)?),
            Some("static_cast<std::uint16_t>(0x1FFULL)".to_owned(),),
        );

        Ok(())
    }

    #[test]
    fn omits_mask_for_full_signed_storage_width() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(SignalType::I8.mask_literal(width(8)?), None,);

        assert_eq!(SignalType::I64.mask_literal(width(64)?), None,);

        Ok(())
    }
}
