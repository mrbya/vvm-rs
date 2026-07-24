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
//! ```
//! use vvm_core::{Bin, Coverpoint, Cross2};
//!
//! let mut opcode = Coverpoint::builder("opcode")
//!     .bin(Bin::value("read", 1_u8))
//!     .build()?;
//! let mut response = Coverpoint::builder("response")
//!     .bin(Bin::value("okay", true))
//!     .build()?;
//! let mut cross = Cross2::builder("opcode_x_response", &opcode, &response).build()?;
//!
//! let opcode_sample = opcode.sample(&1)?;
//! let response_sample = response.sample(&true)?;
//! let cross_sample = cross.sample(&opcode_sample, &response_sample)?;
//!
//! assert!(cross_sample.hit());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Matching precedence is illegal, then ignore, then normal, then unmatched.
//! All matching bins in the selected category increment; lower-precedence bins
//! do not. Illegal hits are recorded before their error is returned. Counter
//! overflow errors leave all counters unchanged.
//!
//! A [`Cross2`] explicitly combines successful samples from two coverpoints.
//! It captures normal-bin definitions at construction, validates the exact
//! producing coverpoint instances, and increments the Cartesian product of
//! overlapping normal bins in row-major order. Ignored or unmatched axes skip
//! the cross; illegal coverpoint samples return before cross sampling. Crosses
//! have a bounded generated-bin cardinality and use exact [`CoverageRatio`]s.
//! Persistence, merging, and reporting are deliberately deferred.
//! Coverage groups are user-owned structs implementing [`CoverageGroup`]. They
//! expose typed coverpoints and crosses through deterministic read-only
//! visitation; sampling remains a concrete method on the user type.
/// Persisted coverage artifacts.
pub mod artifact;
/// Coverage persistence errors.
pub mod artifact_error;
/// Coverage bin model.
pub mod bin;
/// Coverpoint runtime.
pub mod coverpoint;
/// Explicit two-way functional cross coverage.
pub mod cross;
/// Structured cross coverage errors.
pub mod cross_error;
/// Structured coverage errors.
pub mod error;
/// Stable structural coverage fingerprints.
pub mod fingerprint;
/// User-defined typed coverage groups.
pub mod group;
/// Structured coverage-group errors.
pub mod group_error;
/// Shared coverage identifier validation.
mod identifier;
/// Read-only type-erased coverage item inspection.
pub mod item;
/// Declarative matcher representation and validation.
pub mod matcher;
/// Deterministic offline coverage merging.
pub mod merge;
/// Structured coverage merge errors.
pub mod merge_error;
/// Exact coverage ratio.
pub mod ratio;
/// Mutable per-test coverage-session collection.
pub mod session;
/// Structured coverage-session errors.
pub mod session_error;
/// Immutable owned coverage snapshots.
pub mod snapshot;

pub use artifact::{CoverageArtifact, CoverageArtifactGroup};
pub use artifact_error::{CoverageIoOperation, CoveragePersistenceError};
pub use bin::{Bin, BinId, BinKind, CoverpointBin};
pub use coverpoint::{
    CoverageCounterKind, CoverageSampleDisposition, Coverpoint, CoverpointBuilder, CoverpointSample,
};
pub use cross::{Cross2, Cross2Builder, CrossBin, CrossBinId, CrossSample, CrossSampleDisposition};
pub use cross_error::{CrossAxis, CrossBuildError, CrossCounterKind, CrossSampleError};
pub use error::{CoverageBuildError, CoverageSampleError};
pub use fingerprint::CoverageDefinitionFingerprint;
pub use group::{CoverageGroup, CoverageGroupInstance, CoverageGroupSummary, CoverageGroupVisitor};
pub use group_error::{CoverageGroupCountKind, CoverageGroupError};
pub use item::{CoverageItemKind, CoverageItemRef};
pub use matcher::BinMatcherKind;
use matcher::MatcherValidationError;
pub use merge::{CoverageMerge, CoverageMergeInput, CoverageMergePolicy, CoverageMergeSummary};
pub use merge_error::{CoverageMergeCountKind, CoverageMergeCounterKind, CoverageMergeError};
pub use ratio::CoverageRatio;
pub use session::{CoverageSession, CoverageSessionSnapshot, CoverageSessionSummary};
pub use session_error::{CoverageSessionCountKind, CoverageSessionError};
pub use snapshot::{
    CoverageGroupSnapshot, CoverageItemSnapshot, CoverpointBinSnapshot, CoverpointSnapshot,
    Cross2Snapshot, CrossBinSnapshot,
};
