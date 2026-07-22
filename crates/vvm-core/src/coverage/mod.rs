/// Coverage bin model.
pub mod bin;
/// Coverpoint runtime.
pub mod coverpoint;
/// Structured coverage errors.
pub mod error;
/// Declarative matcher representation and validation.
pub mod matcher;

pub use bin::{Bin, BinId, BinKind, CoverpointBin};
pub use coverpoint::{
    CoverageCounterKind, CoverageSampleDisposition, Coverpoint, CoverpointBuilder, CoverpointSample,
};
pub use error::{CoverageBuildError, CoverageSampleError};
pub use matcher::{BinMatcher, MatcherValidationError};
