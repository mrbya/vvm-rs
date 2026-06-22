/// Verilator metadata model definitions.
pub mod model;
/// Ingested Verilator json metadata normalization.
pub mod normalize;
/// Raw Verilator json metadata ingestion.
pub mod raw;

// Re-exports
pub use raw::RawMetadata;
