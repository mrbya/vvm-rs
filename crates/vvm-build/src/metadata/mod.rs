/// Stable VVM-owned DUT metadata model definitions.
pub mod model;
/// Ingested Verilator json metadata normalization.
pub mod normalize;
/// Raw Verilator json metadata ingestion.
pub mod raw;

// Re-exports
pub use model::{BitWidth, DutMetadata, Port, PortDirection};
pub use normalize::{normalize, validate_supported};
pub use raw::RawMetadata;
