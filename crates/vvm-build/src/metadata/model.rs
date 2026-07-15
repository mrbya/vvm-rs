use std::num::NonZeroU32;

/// Non-zero packed bit-width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitWidth(NonZeroU32);

impl BitWidth {
    /// Creates a bit width from a non-zeru value.
    pub const fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    /// Returns the width in bits.
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// Normalized metadata for one DUT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DutMetadata {
    /// Logical VVM DUT name.
    pub name: String,

    /// Elaborated HDL top-module name.
    pub top_module: String,

    /// Top-level ports in HDL declaration order.
    pub ports: Vec<Port>,
}

/// Normalized HDL shape of one top-level port.
///
/// `Port::width` and `Port::signed` remain flattened compatibility fields used
/// by existing scalar and wide codegen. `shape` preserves aggregate structure
/// for later typed aggregate APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortShape {
    /// Plain packed scalar/integral value.
    PackedScalar(PackedScalarShape),

    /// Packed array, still represented by one packed bit-vector at the HDL
    /// boundary.
    PackedArray(PackedArrayShape),

    /// Packed struct with named packed fields.
    PackedStruct(PackedStructShape),

    /// Packed enum with named variants.
    PackedEnum(PackedEnumShape),

    /// Unpacked array. Unlike packed aggregates, this is not one packed scalar
    /// value at the HDL boundary.
    UnpackedArray(UnpackedArrayShape),
}

impl PortShape {
    /// Returns whether this shape is a plain scalar shape currently supported by
    /// scalar/wide codegen.
    #[must_use]
    pub const fn is_plain_packed_scalar(&self) -> bool {
        matches!(self, Self::PackedScalar(_))
    }

    /// Returns whether this shape is any aggregate.
    #[must_use]
    pub const fn is_aggregate(&self) -> bool {
        !self.is_plain_packed_scalar()
    }

    /// Returns the flattened packed width when the shape has one.
    #[must_use]
    pub const fn packed_width(&self) -> Option<BitWidth> {
        match *self {
            Self::PackedScalar(ref shape) => Some(shape.width),
            Self::PackedArray(ref shape) => Some(shape.width),
            Self::PackedStruct(ref shape) => Some(shape.width),
            Self::PackedEnum(ref shape) => Some(shape.width),
            Self::UnpackedArray(_) => None,
        }
    }

    /// Returns flattened signedness for packed shapes.
    #[must_use]
    pub const fn signed(&self) -> bool {
        match *self {
            Self::PackedScalar(ref shape) => shape.signed,
            Self::PackedArray(ref shape) => shape.signed,
            Self::PackedStruct(ref shape) => shape.signed,
            Self::PackedEnum(ref shape) => shape.signed,
            Self::UnpackedArray(_) => false,
        }
    }
}

/// Plain packed scalar shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedScalarShape {
    /// Total packed width.
    pub width: BitWidth,

    /// Whether the scalar is signed.
    pub signed: bool,
}

/// One packed or unpacked array dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayDimension {
    /// Left HDL bound.
    pub left: i64,

    /// Right HDL bound.
    pub right: i64,

    /// Number of elements.
    pub length: NonZeroU32,
}

/// Packed array shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedArrayShape {
    /// Element shape.
    pub element: Box<PortShape>,

    /// Packed dimensions, outermost to innermost as represented by Verilator.
    pub dimensions: Vec<ArrayDimension>,

    /// Flattened packed width.
    pub width: BitWidth,

    /// Whether the aggregate is signed.
    pub signed: bool,
}

/// One packed struct field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedStructField {
    /// Field name.
    pub name: String,

    /// Field shape.
    pub shape: PortShape,

    /// Bit offset from least-significant bit of the packed struct.
    ///
    /// This should be derived from actual Verilator metadata or verified
    /// against emitted accessor behavior, not guessed.
    pub lsb_offset: u32,

    /// Field width.
    pub width: BitWidth,

    /// Field signedness.
    pub signed: bool,
}

/// Packed struct shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedStructShape {
    /// Flattened packed width.
    pub width: BitWidth,

    /// Struct fields.
    pub fields: Vec<PackedStructField>,

    /// Whether the packed struct itself is signed.
    pub signed: bool,
}

/// One enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEnumVariant {
    /// HDL variant name.
    pub name: String,

    /// Raw discriminant value, preserved as a bit pattern.
    pub value: u64,
}

/// Packed enum shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEnumShape {
    /// Enum storage width.
    pub width: BitWidth,

    /// Whether the enum storage type is signed.
    pub signed: bool,

    /// Named variants.
    pub variants: Vec<PackedEnumVariant>,
}

/// Unpacked array shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackedArrayShape {
    /// Element shape.
    pub element: Box<PortShape>,

    /// Unpacked dimensions.
    pub dimensions: Vec<ArrayDimension>,
}

/// One normalized top-level DUT port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    /// HDL-visible port name.
    pub name: String,

    /// HDL port direction.
    pub direction: PortDirection,

    /// Total flattened packed width.
    pub width: BitWidth,

    /// Flattened signedness.
    pub signed: bool,

    /// Structured HDL shape.
    pub shape: PortShape,
}

/// HDL port direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    /// Value that is driven into the DUT.
    Input,

    /// Valua that is sampled from the DUT,
    Output,

    /// Bidirectional port.
    Inout,
}
