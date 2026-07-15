/// Stable VVM-owned DUT metadata model definitions.
pub mod model;
/// Ingested Verilator json metadata normalization.
pub mod normalize;
/// Raw Verilator json metadata ingestion.
pub mod raw;

// Re-exports
pub use model::{
    ArrayDimension, BitWidth, DutMetadata, PackedArrayShape, PackedEnumShape, PackedEnumVariant,
    PackedScalarShape, PackedStructField, PackedStructShape, Port, PortDirection, PortShape,
    UnpackedArrayShape,
};
pub use normalize::{normalize, validate_supported};
pub use raw::RawMetadata;
