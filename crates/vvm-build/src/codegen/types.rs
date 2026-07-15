use crate::metadata::{
    BitWidth, DutMetadata, PackedArrayShape, PackedScalarShape, PackedStructField,
    PackedStructShape, Port, PortShape,
};

/// Public generated type used by adapter methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    /// Scalar port represented by a primitive integer or bool.
    Scalar(SignalType),

    /// Wide port represented by VVM bit-vector wrappers.
    Wide(WideType),
}

/// Supported generated packed-array type descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedArrayType<'a> {
    /// Original normalized port.
    port: &'a Port,

    /// Packed-array shape.
    shape: &'a PackedArrayShape,

    /// Scalar element shape.
    element: &'a PackedScalarShape,

    /// Single supported packed dimension.
    dimension: &'a crate::metadata::ArrayDimension,
}

/// Supported generated packed-struct type descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedStructType<'a> {
    /// Original normalized port.
    port: &'a Port,

    /// Packed-struct shape.
    shape: &'a PackedStructShape,
}

/// Supported packed-struct field descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedStructFieldType<'a> {
    /// Original normalized field.
    field: &'a PackedStructField,

    /// Scalar field shape.
    scalar: &'a PackedScalarShape,
}

/// Generated Rust representation of one packed-struct field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackedStructFieldValueType {
    /// Primitive scalar field.
    Scalar(SignalType),

    /// Wide packed scalar field.
    Wide(WideType),
}

impl PortType {
    /// Selects the generated type for a normalized port.
    pub const fn from_port(port: &Port) -> Self {
        if port.width.get() <= 64 {
            Self::Scalar(SignalType::from_scalar_port(port))
        } else {
            Self::Wide(WideType::new(port.width, port.signed))
        }
    }

    /// Returns whether the selected generated type is wide.
    pub const fn is_wide(self) -> bool {
        matches!(self, Self::Wide(_))
    }
}

impl<'a> PackedArrayType<'a> {
    /// Returns a supported packed-array descriptor for one normalized port.
    pub fn from_port(port: &'a Port) -> Option<Self> {
        let shape = match port.shape {
            PortShape::PackedArray(ref shape) => shape,
            PortShape::PackedScalar(_)
            | PortShape::PackedStruct(_)
            | PortShape::PackedEnum(_)
            | PortShape::UnpackedArray(_) => {
                return None;
            }
        };

        if shape.dimensions.len() != 1 {
            return None;
        }

        let dimension = shape.dimensions.first()?;

        let element = match *shape.element.as_ref() {
            PortShape::PackedScalar(ref element) => element,
            PortShape::PackedArray(_)
            | PortShape::PackedStruct(_)
            | PortShape::PackedEnum(_)
            | PortShape::UnpackedArray(_) => {
                return None;
            }
        };

        if element.width.get() > 64 {
            return None;
        }

        Some(Self {
            port,
            shape,
            element,
            dimension,
        })
    }

    /// Returns the total flattened packed width.
    pub const fn total_width(self) -> u32 {
        self.port.width.get()
    }

    /// Returns the storage signedness.
    pub const fn storage_signed(self) -> bool {
        self.port.signed
    }

    /// Returns the storage Rust value type.
    pub fn storage_rust_type(self) -> String {
        WideType::new(self.port.width, self.port.signed).rust_value_type()
    }

    /// Returns the storage Rust constructor type.
    pub fn storage_constructor_type(self) -> String {
        WideType::new(self.port.width, self.port.signed).rust_constructor_type()
    }

    /// Returns the generated element Rust scalar type.
    pub const fn element_rust_type(self) -> &'static str {
        self.element_signal_type().rust_type()
    }

    /// Returns the scalar extraction helper path.
    pub const fn extraction_function(self) -> &'static str {
        if self.element.signed {
            "::vvm::extract_signed"
        } else {
            "::vvm::extract_unsigned"
        }
    }

    /// Returns the scalar insertion helper path.
    pub const fn insertion_function(self) -> &'static str {
        if self.element.signed {
            "::vvm::insert_signed"
        } else {
            "::vvm::insert_unsigned"
        }
    }

    /// Returns one element width in bits.
    pub const fn element_width(self) -> u32 {
        self.element.width.get()
    }

    /// Returns the left HDL index bound.
    pub const fn left(self) -> i64 {
        self.dimension.left
    }

    /// Returns the right HDL index bound.
    pub const fn right(self) -> i64 {
        self.dimension.right
    }

    /// Returns the element count.
    pub const fn length(self) -> u32 {
        self.dimension.length.get()
    }

    /// Returns whether elements are signed.
    pub const fn element_signed(self) -> bool {
        self.element.signed
    }

    /// Returns whether elements are represented as bool.
    pub const fn element_is_bool(self) -> bool {
        !self.element.signed && self.element.width.get() == 1
    }

    /// Returns the element signal type mapping.
    pub const fn element_signal_type(self) -> SignalType {
        SignalType::from_width_signed(self.element.width.get(), self.element.signed)
    }

    /// Returns the flattened native transfer representation.
    pub const fn storage_port_type(self) -> PortType {
        PortType::from_port(self.port)
    }
}

impl<'a> PackedStructType<'a> {
    /// Returns a supported packed-struct descriptor.
    #[allow(clippy::pattern_type_mismatch)]
    pub fn from_port(port: &'a Port) -> Option<Self> {
        let PortShape::PackedStruct(shape) = &port.shape else {
            return None;
        };

        if shape.fields.is_empty() {
            return None;
        }

        if shape
            .fields
            .iter()
            .any(|field| !matches!(field.shape, PortShape::PackedScalar(_)))
        {
            return None;
        }

        Some(Self { port, shape })
    }

    /// Returns the total packed width.
    pub const fn total_width(self) -> u32 {
        self.shape.width.get()
    }

    /// Returns the storage signedness.
    pub const fn storage_signed(self) -> bool {
        self.shape.signed
    }

    /// Returns the generated Rust storage type.
    pub fn storage_rust_type(self) -> String {
        WideType::new(self.shape.width, self.shape.signed).rust_value_type()
    }

    /// Returns the generated Rust storage constructor type.
    pub fn storage_constructor_type(self) -> String {
        WideType::new(self.shape.width, self.shape.signed).rust_constructor_type()
    }

    /// Returns the flattened native transfer type.
    pub const fn storage_port_type(self) -> PortType {
        PortType::from_port(self.port)
    }

    /// Returns the normalized packed-struct shape.
    pub const fn shape(self) -> &'a PackedStructShape {
        self.shape
    }

    /// Returns generated field descriptors.
    pub fn fields(self) -> impl Iterator<Item = PackedStructFieldType<'a>> {
        self.shape
            .fields
            .iter()
            .filter_map(PackedStructFieldType::from_field)
    }
}

impl<'a> PackedStructFieldType<'a> {
    /// Creates a descriptor for one scalar packed-struct field.
    #[allow(clippy::pattern_type_mismatch)]
    pub const fn from_field(field: &'a PackedStructField) -> Option<Self> {
        let PortShape::PackedScalar(scalar) = &field.shape else {
            return None;
        };

        Some(Self { field, scalar })
    }

    /// Returns the HDL field name.
    pub fn name(self) -> &'a str {
        &self.field.name
    }

    /// Returns the field width.
    pub const fn width(self) -> u32 {
        self.scalar().width.get()
    }

    /// Returns the field signedness.
    pub const fn signed(self) -> bool {
        self.scalar().signed
    }

    /// Returns the field LSB offset.
    pub const fn offset(self) -> u32 {
        self.field.lsb_offset
    }

    /// Returns whether this is an unsigned one-bit field.
    pub const fn is_bool(self) -> bool {
        !self.signed() && self.width() == 1
    }

    /// Returns the generated field representation.
    pub const fn value_type(self) -> PackedStructFieldValueType {
        if self.width() <= 64 {
            PackedStructFieldValueType::Scalar(SignalType::from_width_signed(
                self.width(),
                self.signed(),
            ))
        } else {
            PackedStructFieldValueType::Wide(WideType::new(self.field.width, self.signed()))
        }
    }

    /// Returns the normalized scalar shape.
    pub const fn scalar(self) -> &'a PackedScalarShape {
        self.scalar
    }
}

/// Public wide type used by generated adapter methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WideType {
    /// Total packed width in bits.
    width: BitWidth,

    /// Whether the public Rust wrapper uses a signed bit-vector type.
    signed: bool,
}

impl WideType {
    /// Creates a wide type description.
    pub const fn new(width: BitWidth, signed: bool) -> Self {
        Self { width, signed }
    }

    /// Returns the packed bit width.
    pub const fn width(self) -> BitWidth {
        self.width
    }

    /// Returns whether the wide type is signed.
    pub const fn signed(self) -> bool {
        self.signed
    }

    /// Returns the number of little-endian transfer words.
    pub const fn word_count(self) -> u32 {
        self.width.get().div_ceil(32)
    }

    /// Returns the generated Rust value type.
    pub fn rust_value_type(self) -> String {
        let width = self.width().get();

        if self.signed() {
            format!("::vvm::SignedBits<{width}>")
        } else {
            format!("::vvm::Bits<{width}>")
        }
    }

    /// Returns the generated Rust constructor type.
    pub fn rust_constructor_type(self) -> String {
        let width = self.width().get();

        if self.signed() {
            format!("::vvm::SignedBits::<{width}>")
        } else {
            format!("::vvm::Bits::<{width}>")
        }
    }

    /// Returns the mask for the final transfer word.
    pub const fn final_word_mask(self) -> u32 {
        let remainder = self.width().get() % 32;

        if remainder == 0 {
            u32::MAX
        } else {
            match u32::MAX.checked_shr(u32::BITS.saturating_sub(remainder)) {
                Some(mask) => mask,
                None => 0,
            }
        }
    }

    /// Returns whether the final transfer word needs masking.
    pub const fn requires_final_word_mask(self) -> bool {
        !self.width().get().is_multiple_of(32)
    }

    /// Returns the C++ literal for the final word mask.
    pub fn final_word_mask_literal(self) -> String {
        format!("0x{:X}U", self.final_word_mask())
    }
}

/// Returns whether any generated DUT port needs wide transfer support.
pub fn contains_wide_ports(metadata: &DutMetadata) -> bool {
    metadata
        .ports
        .iter()
        .any(|port| PortType::from_port(port).is_wide())
}

/// Returns whether any port uses a supported packed-array wrapper.
pub fn contains_packed_array_ports(metadata: &DutMetadata) -> bool {
    metadata
        .ports
        .iter()
        .any(|port| PackedArrayType::from_port(port).is_some())
}

/// Returns whether any port uses a supported packed-struct wrapper.
pub fn contains_packed_struct_ports(metadata: &DutMetadata) -> bool {
    metadata
        .ports
        .iter()
        .any(|port| PackedStructType::from_port(port).is_some())
}

/// Returns whether any port uses a supported packed aggregate wrapper.
pub fn contains_packed_aggregate_ports(metadata: &DutMetadata) -> bool {
    contains_packed_array_ports(metadata) || contains_packed_struct_ports(metadata)
}

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
    /// Selects the public scalar type for one width/signedness pair.
    pub const fn from_width_signed(width: u32, signed: bool) -> Self {
        match (signed, width) {
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

    /// Selects the public scalar type for a normalized port.
    pub const fn from_scalar_port(port: &Port) -> Self {
        Self::from_width_signed(port.width.get(), port.signed)
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

    use super::{
        PackedArrayType, PackedStructFieldValueType, PackedStructType, PortType, SignalType,
        WideType, contains_packed_aggregate_ports, contains_packed_array_ports,
        contains_packed_struct_ports, contains_wide_ports,
    };
    use crate::metadata::{
        ArrayDimension, BitWidth, PackedArrayShape, PackedScalarShape, PackedStructField,
        PackedStructShape, Port, PortDirection, PortShape,
    };

    fn width(value: u32) -> Result<BitWidth, io::Error> {
        NonZeroU32::new(value).map(BitWidth::new).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "test width must be non-zero")
        })
    }

    fn signal_type(bits: u32, signed: bool) -> Result<SignalType, io::Error> {
        let width = width(bits)?;
        let port = Port {
            name: String::from("value"),
            direction: PortDirection::Input,
            width,
            signed,
            shape: PortShape::PackedScalar(PackedScalarShape { width, signed }),
        };

        Ok(SignalType::from_scalar_port(&port))
    }

    fn port_type(bits: u32, signed: bool) -> Result<PortType, io::Error> {
        let width = width(bits)?;
        let port = Port {
            name: String::from("value"),
            direction: PortDirection::Input,
            width,
            signed,
            shape: PortShape::PackedScalar(PackedScalarShape { width, signed }),
        };

        Ok(PortType::from_port(&port))
    }

    fn wide_type(bits: u32, signed: bool) -> Result<WideType, io::Error> {
        match port_type(bits, signed)? {
            PortType::Wide(wide) => Ok(wide),
            PortType::Scalar(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected wide test type",
            )),
        }
    }

    fn packed_array_port(
        width_bits: u32,
        signed: bool,
        element_width_bits: u32,
        element_signed: bool,
        dimensions: Vec<ArrayDimension>,
    ) -> Result<Port, io::Error> {
        let total_width = width(width_bits)?;
        let element_width = width(element_width_bits)?;

        Ok(Port {
            name: String::from("packed"),
            direction: PortDirection::Input,
            width: total_width,
            signed,
            shape: PortShape::PackedArray(PackedArrayShape {
                element: Box::new(PortShape::PackedScalar(PackedScalarShape {
                    width: element_width,
                    signed: element_signed,
                })),
                dimensions,
                width: total_width,
                signed,
            }),
        })
    }

    fn packed_struct_port(
        width_bits: u32,
        signed: bool,
        fields: Vec<PackedStructField>,
    ) -> Result<Port, io::Error> {
        let total_width = width(width_bits)?;

        Ok(Port {
            name: String::from("packet"),
            direction: PortDirection::Input,
            width: total_width,
            signed,
            shape: PortShape::PackedStruct(PackedStructShape {
                width: total_width,
                fields,
                signed,
            }),
        })
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

    #[test]
    fn sixty_four_bit_ports_remain_scalar() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(port_type(64, false)?, PortType::Scalar(SignalType::U64));
        assert_eq!(port_type(64, true)?, PortType::Scalar(SignalType::I64));

        Ok(())
    }

    #[test]
    fn sixty_five_bit_ports_become_wide() -> Result<(), Box<dyn std::error::Error>> {
        assert!(port_type(65, false)?.is_wide());
        assert!(port_type(65, true)?.is_wide());

        Ok(())
    }

    #[test]
    fn unsigned_wide_maps_to_bits() -> Result<(), Box<dyn std::error::Error>> {
        let wide = wide_type(65, false)?;

        assert_eq!(wide.rust_value_type(), "::vvm::Bits<65>");
        assert_eq!(wide.rust_constructor_type(), "::vvm::Bits::<65>");

        Ok(())
    }

    #[test]
    fn signed_wide_maps_to_signed_bits() -> Result<(), Box<dyn std::error::Error>> {
        let wide = wide_type(129, true)?;

        assert_eq!(wide.rust_value_type(), "::vvm::SignedBits<129>");
        assert_eq!(wide.rust_constructor_type(), "::vvm::SignedBits::<129>");

        Ok(())
    }

    #[test]
    fn sixty_five_bit_wide_type_uses_three_words_and_single_bit_mask()
    -> Result<(), Box<dyn std::error::Error>> {
        let wide = wide_type(65, false)?;

        assert_eq!(wide.word_count(), 3);
        assert_eq!(wide.final_word_mask(), 0x0000_0001);
        assert!(wide.requires_final_word_mask());
        assert_eq!(wide.final_word_mask_literal(), "0x1U");

        Ok(())
    }

    #[test]
    fn ninety_six_bit_wide_type_uses_full_final_word() -> Result<(), Box<dyn std::error::Error>> {
        let wide = wide_type(96, false)?;

        assert_eq!(wide.word_count(), 3);
        assert_eq!(wide.final_word_mask(), 0xffff_ffff);
        assert!(!wide.requires_final_word_mask());
        assert_eq!(wide.final_word_mask_literal(), "0xFFFFFFFFU");

        Ok(())
    }

    #[test]
    fn one_hundred_twenty_nine_bit_wide_type_uses_five_words()
    -> Result<(), Box<dyn std::error::Error>> {
        let wide = wide_type(129, true)?;

        assert_eq!(wide.word_count(), 5);
        assert_eq!(wide.final_word_mask(), 0x0000_0001);
        assert!(wide.requires_final_word_mask());

        Ok(())
    }

    #[test]
    fn signedness_does_not_change_wide_word_layout() -> Result<(), Box<dyn std::error::Error>> {
        let unsigned = wide_type(129, false)?;
        let signed = wide_type(129, true)?;

        assert_eq!(unsigned.word_count(), signed.word_count());
        assert_eq!(unsigned.final_word_mask(), signed.final_word_mask());
        assert_eq!(
            unsigned.requires_final_word_mask(),
            signed.requires_final_word_mask()
        );

        Ok(())
    }

    #[test]
    fn detects_metadata_with_wide_ports() -> Result<(), Box<dyn std::error::Error>> {
        let scalar = crate::metadata::DutMetadata {
            name: "scalar".to_owned(),
            top_module: "scalar".to_owned(),
            ports: vec![{
                let width = width(64)?;

                Port {
                    name: "value".to_owned(),
                    direction: PortDirection::Input,
                    width,
                    signed: false,
                    shape: PortShape::PackedScalar(PackedScalarShape {
                        width,
                        signed: false,
                    }),
                }
            }],
        };
        let wide = crate::metadata::DutMetadata {
            name: "wide".to_owned(),
            top_module: "wide".to_owned(),
            ports: vec![{
                let width = width(65)?;

                Port {
                    name: "value".to_owned(),
                    direction: PortDirection::Input,
                    width,
                    signed: false,
                    shape: PortShape::PackedScalar(PackedScalarShape {
                        width,
                        signed: false,
                    }),
                }
            }],
        };

        assert!(!contains_wide_ports(&scalar));
        assert!(contains_wide_ports(&wide));

        Ok(())
    }

    #[test]
    fn detects_supported_packed_array_ports() -> Result<(), Box<dyn std::error::Error>> {
        let metadata = crate::metadata::DutMetadata {
            name: String::from("packed"),
            top_module: String::from("packed"),
            ports: vec![packed_array_port(
                32,
                false,
                8,
                false,
                vec![ArrayDimension {
                    left: 3,
                    right: 0,
                    length: NonZeroU32::new(4).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "test dimension length")
                    })?,
                }],
            )?],
        };

        assert!(contains_packed_array_ports(&metadata));

        Ok(())
    }

    #[test]
    fn describes_supported_packed_array_type() -> Result<(), Box<dyn std::error::Error>> {
        let port = packed_array_port(
            32,
            false,
            8,
            false,
            vec![ArrayDimension {
                left: 3,
                right: 0,
                length: NonZeroU32::new(4).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "test dimension length")
                })?,
            }],
        )?;

        let array_type = PackedArrayType::from_port(&port).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "expected supported packed array",
            )
        })?;

        assert_eq!(array_type.storage_rust_type(), "::vvm::Bits<32>");
        assert_eq!(array_type.storage_constructor_type(), "::vvm::Bits::<32>");
        assert_eq!(array_type.element_rust_type(), "u8");
        assert_eq!(array_type.extraction_function(), "::vvm::extract_unsigned");
        assert_eq!(array_type.insertion_function(), "::vvm::insert_unsigned");
        assert_eq!(array_type.element_width(), 8);
        assert_eq!(array_type.total_width(), 32);
        assert_eq!(array_type.left(), 3);
        assert_eq!(array_type.right(), 0);
        assert_eq!(array_type.length(), 4);
        assert_eq!(
            array_type.storage_port_type(),
            PortType::Scalar(SignalType::U32)
        );

        Ok(())
    }

    #[test]
    fn detects_supported_packed_struct_ports() -> Result<(), Box<dyn std::error::Error>> {
        let metadata = crate::metadata::DutMetadata {
            name: String::from("packed"),
            top_module: String::from("packed"),
            ports: vec![packed_struct_port(
                16,
                false,
                vec![
                    PackedStructField {
                        name: String::from("low"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(8)?,
                            signed: false,
                        }),
                        lsb_offset: 0,
                        width: width(8)?,
                        signed: false,
                    },
                    PackedStructField {
                        name: String::from("high"),
                        shape: PortShape::PackedScalar(PackedScalarShape {
                            width: width(8)?,
                            signed: false,
                        }),
                        lsb_offset: 8,
                        width: width(8)?,
                        signed: false,
                    },
                ],
            )?],
        };

        assert!(contains_packed_struct_ports(&metadata));
        assert!(contains_packed_aggregate_ports(&metadata));

        Ok(())
    }

    #[test]
    fn describes_supported_packed_struct_type() -> Result<(), Box<dyn std::error::Error>> {
        let port = packed_struct_port(
            136,
            false,
            vec![
                PackedStructField {
                    name: String::from("tag"),
                    shape: PortShape::PackedScalar(PackedScalarShape {
                        width: width(7)?,
                        signed: false,
                    }),
                    lsb_offset: 129,
                    width: width(7)?,
                    signed: false,
                },
                PackedStructField {
                    name: String::from("payload"),
                    shape: PortShape::PackedScalar(PackedScalarShape {
                        width: width(129)?,
                        signed: true,
                    }),
                    lsb_offset: 0,
                    width: width(129)?,
                    signed: true,
                },
            ],
        )?;

        let struct_type = PackedStructType::from_port(&port).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "expected supported packed struct",
            )
        })?;

        assert_eq!(struct_type.total_width(), 136);
        assert!(!struct_type.storage_signed());
        assert_eq!(struct_type.storage_rust_type(), "::vvm::Bits<136>");
        assert_eq!(struct_type.storage_constructor_type(), "::vvm::Bits::<136>");
        assert_eq!(
            struct_type.storage_port_type(),
            PortType::Wide(WideType::new(width(136)?, false))
        );

        let fields: Vec<_> = struct_type.fields().collect();
        let tag = fields
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing tag field"))?;
        let payload = fields
            .get(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing payload field"))?;

        assert_eq!(fields.len(), 2);
        assert_eq!(tag.name(), "tag");
        assert_eq!(tag.offset(), 129);
        assert_eq!(tag.width(), 7);
        assert!(!tag.signed());
        assert!(!tag.is_bool());
        assert_eq!(tag.scalar().width.get(), 7);
        assert_eq!(
            tag.value_type(),
            PackedStructFieldValueType::Scalar(SignalType::U8)
        );

        assert_eq!(payload.name(), "payload");
        assert_eq!(payload.offset(), 0);
        assert_eq!(payload.width(), 129);
        assert!(payload.signed());
        assert_eq!(payload.scalar().width.get(), 129);
        assert_eq!(
            payload.value_type(),
            PackedStructFieldValueType::Wide(WideType::new(width(129)?, true))
        );

        Ok(())
    }

    #[test]
    fn packed_struct_bool_field_maps_to_bool() -> Result<(), Box<dyn std::error::Error>> {
        let port = packed_struct_port(
            1,
            false,
            vec![PackedStructField {
                name: String::from("valid"),
                shape: PortShape::PackedScalar(PackedScalarShape {
                    width: width(1)?,
                    signed: false,
                }),
                lsb_offset: 0,
                width: width(1)?,
                signed: false,
            }],
        )?;

        let field = PackedStructType::from_port(&port)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected packed struct"))?
            .fields()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing field"))?;

        assert!(field.is_bool());
        assert_eq!(
            field.value_type(),
            PackedStructFieldValueType::Scalar(SignalType::Bool)
        );

        Ok(())
    }
}
