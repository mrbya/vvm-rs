//! Pure Rust verification primitives for VVM.

#![allow(clippy::module_name_repetitions)]
// clippy WARN level lints
#![warn(
    missing_docs,
    //clippy::cargo,
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

/// Synchronous clock-driving abstraction.
pub(crate) mod clock;
/// Stimulus driving abstraction.
pub(crate) mod drive;
/// DUT lifecycle abstraction.
pub(crate) mod dut;
/// Bounded check-failure policy.
pub(crate) mod failure_policy;
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
/// Reusable synchronous testbench runner.
pub(crate) mod testbench;
/// Explicit simulation time.
pub(crate) mod time;

// Re-exports
pub use clock::Clock;
pub use drive::Drive;
pub use dut::{Dut, TraceableDut};
pub use failure_policy::{FailurePolicy, InvalidFailureLimit};
pub use random::{
    ParseReplayTokenError, RandomAlgorithm, RandomContext, Randomize, ReplayToken,
    ReplayableSequence, Seed,
};
pub use reference_model::ReferenceModel;
pub use registry::{
    TestDescriptor, TestFunction, TestKind, TestOutcome, TestRegistry, TestRegistryError, TestRun,
    TestRunConfig, TestStatistics, TestStatus,
};
pub use report::{DetailedTestReport, TestSummary};
pub use result::{CheckFailure, SimulationError, SimulationStage, TestResult};
pub use sample::Sample;
pub use scoreboard::{ExactScoreboard, Mismatch, Scoreboard};
pub use testbench::{Testbench, Unconfigured};
pub use time::{CycleTiming, InvalidTimeStep, SimulationTime, TimeStep};

// Unit tests.
#[cfg(test)]
mod tests;
