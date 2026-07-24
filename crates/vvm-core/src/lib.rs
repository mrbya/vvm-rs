//! Pure Rust verification primitives for VVM.

#![allow(clippy::module_name_repetitions)]
// clippy WARN level lints
#![warn(
    missing_docs,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::missing_docs_in_private_items,
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::modulo_arithmetic,
    clippy::as_conversions,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::pattern_type_mismatch,
    clippy::string_slice,
    clippy::try_err
)]
// clippy DENY level lints, they always have a quick fix that should be preferred
#![deny(
    clippy::wildcard_imports,
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::implicit_clone,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

/// Arbitrary-width packed bit values.
pub(crate) mod bits;
/// Synchronous clock-driving abstraction and configuration.
pub(crate) mod clock;
/// Functional coverage primitives.
pub(crate) mod coverage;
/// Stimulus driving abstraction.
pub(crate) mod drive;
/// DUT lifecycle abstraction.
pub(crate) mod dut;
/// Bounded check-failure policy.
pub(crate) mod failure_policy;
/// Raw bidirectional-port state snapshots.
pub(crate) mod inout;
/// Packed aggregate layout helpers.
pub(crate) mod packed;
/// Randomization support.
pub(crate) mod random;
/// Reference model abstraction.
pub(crate) mod reference_model;
/// Simulation test registry.
pub(crate) mod registry;
/// Structured reporting.
pub(crate) mod report;
/// Testbench outcome and diagnostic types.
pub(crate) mod result;
/// DUT observation abstraction.
pub(crate) mod sample;
/// Scoreboard abstraction and exact-equality implementation.
pub(crate) mod scoreboard;
/// Mutable context for context-aware registered tests.
pub(crate) mod test_context;
/// Reusable synchronous testbench runner.
pub(crate) mod testbench;
/// Explicit simulation time.
pub(crate) mod time;
/// Internally scheduled delayed-event execution.
pub(crate) mod timing;
/// Unpacked-array indexing helpers.
pub(crate) mod unpacked;

// Re-exports
pub use bits::{Bits, InvalidBitVectorWordCount, SignedBits};
pub use clock::{Clock, ClockConfigurationError, ClockScheduler, ClockTiming};
pub use coverage::{
    Bin, BinId, BinKind, BinMatcherKind, CoverageArtifact, CoverageArtifactGroup,
    CoverageBinDetail, CoverageBuildError, CoverageCounterKind, CoverageDefinitionFingerprint,
    CoverageGroup, CoverageGroupCountKind, CoverageGroupError, CoverageGroupInstance,
    CoverageGroupSnapshot, CoverageGroupSummary, CoverageGroupVisitor, CoverageIoOperation,
    CoverageItemKind, CoverageItemRef, CoverageItemSnapshot, CoverageMerge, CoverageMergeCountKind,
    CoverageMergeCounterKind, CoverageMergeError, CoverageMergeInput, CoverageMergePolicy,
    CoverageMergeSummary, CoveragePercentage, CoveragePersistenceError, CoverageRatio,
    CoverageReport, CoverageReportOptions, CoverageSampleDisposition, CoverageSampleError,
    CoverageSession, CoverageSessionCountKind, CoverageSessionError, CoverageSessionSnapshot,
    CoverageSessionSummary, Coverpoint, CoverpointBin, CoverpointBinSnapshot, CoverpointBuilder,
    CoverpointSample, CoverpointSnapshot, Cross2, Cross2Builder, Cross2Snapshot, CrossAxis,
    CrossBin, CrossBinId, CrossBinSnapshot, CrossBuildError, CrossCounterKind, CrossSample,
    CrossSampleDisposition, CrossSampleError,
};
pub use drive::Drive;
pub use dut::{Dut, TimedDut, TraceableDut};
pub use failure_policy::{FailurePolicy, InvalidFailureLimit};
pub use inout::InoutState;
pub use packed::{
    PackedEnumLayout, PackedEnumVariantLayout, PackedFieldLayout, PackedLayout, PackedLayoutError,
    PackedRange, PackedValue, extract_packed, extract_signed, extract_signed_packed,
    extract_unsigned, insert_packed, insert_signed, insert_signed_packed, insert_unsigned,
};
pub use random::{
    ParseReplayTokenError, RandomAlgorithm, RandomContext, Randomize, ReplayToken,
    ReplayableSequence, Seed,
};
pub use reference_model::ReferenceModel;
pub use registry::{
    ContextTestFunction, IntoTestOutcome, TestCapabilities, TestDescriptor, TestFunction,
    TestOutcome, TestRegistry, TestRegistryError, TestRun, TestRunConfig, TestStatistics,
    TestStatus,
};
pub use report::{DetailedTestReport, TestSummary};
pub use result::{CheckFailure, SimulationError, SimulationStage, TestResult};
pub use sample::Sample;
pub use scoreboard::{ExactScoreboard, Mismatch, Scoreboard};
pub use test_context::TestContext;
pub use testbench::{ObservedCycle, Testbench, Unconfigured};
pub use time::{CycleTiming, InvalidTimeStep, SimulationTime, TimeStep};
pub use timing::{TimingEvent, TimingRun, TimingScheduler, TimingSchedulerError, TimingStage};
pub use unpacked::{UnpackedArrayIndexError, unpacked_array_ordinal};

// Unit tests.
#[cfg(test)]
mod tests;
