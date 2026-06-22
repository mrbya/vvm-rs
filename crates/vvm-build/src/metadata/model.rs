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

/// One normalized top-level DUT port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    /// HDL-visible port name.
    pub name: String,

    /// HDL port direction.
    pub direction: PortDirection,

    /// Total packed width.
    pub width: BitWidth,

    /// Whether the port's integral datatype is signed.
    pub signed: bool,
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
