//! Explicitly sampled Rust-native functional coverage primitives.
//!
//! A [`Coverpoint`] owns its runtime counters. Define declarative [`Bin`]s,
//! bind them with [`Coverpoint::builder`], and call [`Coverpoint::sample`]
//! explicitly when an observed value is ready. No global registration or test
//! harness integration occurs.
//!
//! ```
//! use vvm_core::{Bin, Coverpoint};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Opcode { Reset, Read, WriteByte, WriteWord, Reserved }
//!
//! let mut opcode = Coverpoint::builder("opcode")
//!     .bin(Bin::value("read", Opcode::Read))
//!     .bin(Bin::values("writes", [Opcode::WriteByte, Opcode::WriteWord]).at_least(2))
//!     .ignore_bin(Bin::value("reset", Opcode::Reset))
//!     .illegal_bin(Bin::value("reserved", Opcode::Reserved))
//!     .build()?;
//!
//! let sample = opcode.sample(&Opcode::Read)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Matching precedence is illegal, then ignore, then normal, then unmatched.
//! All matching bins in the selected category increment; lower-precedence bins
//! do not. Illegal hits are recorded before their error is returned. Counter
//! overflow errors leave all counters unchanged.
//!
//! Only normal bins contribute to [`CoverageRatio`] completion. Percentages,
//! persistence, merging, reporting, and cross coverage are deliberately
//! outside this primitive layer.
/// Coverage bin model.
pub mod bin;
/// Coverpoint runtime.
pub mod coverpoint;
/// Structured coverage errors.
pub mod error;
/// Declarative matcher representation and validation.
pub mod matcher;
/// Exact coverage ratio.
pub mod ratio;

pub use bin::{Bin, BinId, BinKind, CoverpointBin};
pub use coverpoint::{
    CoverageCounterKind, CoverageSampleDisposition, Coverpoint, CoverpointBuilder, CoverpointSample,
};
pub use error::{CoverageBuildError, CoverageSampleError};
use matcher::MatcherValidationError;
pub use ratio::CoverageRatio;
