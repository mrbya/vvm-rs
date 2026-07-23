//! Verilator Verification Methodology for Rust.
//!
//! VVM provides strongly typed Rust testbenches for Verilator-generated HDL
//! models, including explicit execution of internally delayed HDL processes.
//!
//! Tests marked with `#[vvm::test]` become ordinary Rust `#[test]` functions.
//! Place unit-style VVM tests inside an explicit `#[cfg(test)]` module. Cargo
//! integration tests under `tests/` do not need an additional `cfg(test)`.
//! Standard Rust tooling such as `cargo test` and `cargo nextest run` owns test
//! discovery, filtering, and execution.
//!
//! The runtime facade exposes:
//!
//! - generated DUT integration;
//! - typed stimulus driving and output sampling;
//! - typed single-clock and independently timed multi-clock control through
//!   [`Clock`], [`ClockTiming`], [`ClockScheduler`], and [`Testbench::with_clocks`];
//! - explicit execution of internally delayed HDL processes through [`TimedDut`]
//!   and [`TimingScheduler`];
//! - stateful reference models;
//! - scoreboards and structured mismatches;
//! - deterministic testbench execution.
//! - explicitly sampled Rust-native functional coverage through [`Coverpoint`], [`Bin`], and [`Cross2`];
//!
//! Native compilation and DUT generation are configured separately through
//! the `vvm-build` crate from a consuming package's `build.rs`.
//!
//! Functional coverage is owned by ordinary Rust values and sampled explicitly:
//!
//! ```
//! use vvm::{Bin, Coverpoint, Cross2};
//!
//! let mut coverage = Coverpoint::builder("value")
//!     .bin(Bin::value("zero", 0_u8))
//!     .bin(Bin::inclusive_range("nonzero", 1_u8, u8::MAX))
//!     .build()?;
//!
//! assert!(coverage.sample(&0)?.hit());
//! assert_eq!(coverage.coverage().covered(), 1);
//!
//! let mut response = Coverpoint::builder("response")
//!     .bin(Bin::value("okay", true))
//!     .build()?;
//! let mut cross = Cross2::builder("value_x_response", &coverage, &response).build()?;
//! let value = coverage.sample(&0)?;
//! let response = response.sample(&true)?;
//! assert!(cross.sample(&value, &response)?.hit());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Context-aware VVM tests can capture immutable owned coverage snapshots at
//! the end of a test body. `capture_coverage()` freezes the group's state at
//! that call; later sampling does not alter the captured data, and no files are
//! written by this operation.
//!
//! ```ignore
//! #[vvm::test]
//! fn covered(context: &mut vvm::TestContext) -> Result<MyTestResult, Box<dyn std::error::Error>> {
//!     let mut coverage = MyCoverage::new("dut.decoder")?;
//!     let result = run_test_body(context.config(), &mut coverage);
//!     context.capture_coverage(&coverage)?;
//!     result
//! }
//! ```
//!
//! # Example
//!
//! A build script describes the DUT:
//!
//! ```ignore
//! use vvm_build::{BuildResult, DutBuilder};
//!
//! fn main() -> BuildResult<()> {
//!     DutBuilder::new("counter")
//!         .top_module("counter")
//!         .source("rtl/counter.sv")
//!         .build()
//! }
//! ```
//!
//! Timing-enabled models use a separate scheduler rather than the cycle-driven
//! testbench loop:
//!
//! ```ignore
//! let mut scheduler = TimingScheduler::new();
//! let run = scheduler.run_until_idle(&mut dut, max_slots)?;
//! ```
//!
//! The generated DUT can then be included and verified:
//!
//! ```ignore
//! #[cfg(test)]
//! mod vvm_tests {
//!     use vvm::prelude::*;
//!
//!     vvm::include_dut!(counter);
//!
//!     use counter::Counter;
//!
//!     #[derive(Clone, Copy, Debug, Drive)]
//!     #[vvm(dut = Counter)]
//!     struct CounterStimulus {
//!         #[vvm(port)]
//!         reset_n: bool,
//!
//!         #[vvm(port)]
//!         enable: bool,
//!     }
//!
//!     #[derive(Clone, Copy, Debug, PartialEq, Eq, Sample)]
//!     #[vvm(dut = Counter)]
//!     struct CounterObservation {
//!         #[vvm(port)]
//!         count: u8,
//!     }
//!
//!     #[derive(Clone, Copy, Debug, Default, Clock)]
//!     #[vvm(
//!         dut = Counter,
//!         clock = "clk"
//!     )]
//!     struct CounterClock;
//!
//!     /// Counter smoke verification.
//!     #[vvm::test(trace)]
//!     fn counter_smoke(config: &vvm::TestRunConfig) -> Result<CounterTestResult> {
//!         let mut dut = Counter::new()?;
//!
//!         config.configure_trace(&mut dut)?;
//!
//!         Ok(
//!             Testbench::new(dut)
//!                 // ...
//!                 .run::<CounterObservation>(),
//!         )
//!     }
//! }
//! ```

/// Standard Rust test harness integration support.
pub(crate) mod test;

#[doc(inline)]
pub use vvm_core::{
    Bin, BinId, BinKind, Bits, CheckFailure, Clock, ClockConfigurationError, ClockScheduler,
    ClockTiming, ContextTestFunction, CoverageBuildError, CoverageCounterKind, CoverageGroup,
    CoverageGroupCountKind, CoverageGroupError, CoverageGroupInstance, CoverageGroupSnapshot,
    CoverageGroupSummary, CoverageGroupVisitor, CoverageItemKind, CoverageItemRef,
    CoverageItemSnapshot, CoverageRatio, CoverageSampleDisposition, CoverageSampleError,
    CoverageSession, CoverageSessionCountKind, CoverageSessionError, CoverageSessionSnapshot,
    CoverageSessionSummary, Coverpoint, CoverpointBin, CoverpointBinSnapshot, CoverpointBuilder,
    CoverpointSample, CoverpointSnapshot, Cross2, Cross2Builder, Cross2Snapshot, CrossAxis,
    CrossBin, CrossBinId, CrossBinSnapshot, CrossBuildError, CrossCounterKind, CrossSample,
    CrossSampleDisposition, CrossSampleError, CycleTiming, DetailedTestReport, Drive, Dut,
    ExactScoreboard, FailurePolicy, InoutState, IntoTestOutcome, InvalidBitVectorWordCount,
    InvalidFailureLimit, InvalidTimeStep, Mismatch, PackedEnumLayout, PackedEnumVariantLayout,
    PackedFieldLayout, PackedLayout, PackedLayoutError, PackedRange, PackedValue,
    ParseReplayTokenError, RandomAlgorithm, RandomContext, Randomize, ReferenceModel, ReplayToken,
    ReplayableSequence, Sample, Scoreboard, Seed, SignedBits, SimulationError, SimulationStage,
    SimulationTime, TestCapabilities, TestContext, TestDescriptor, TestFunction, TestOutcome,
    TestRegistry, TestRegistryError, TestResult, TestRun, TestRunConfig, TestStatistics,
    TestStatus, TestSummary, Testbench, TimeStep, TimedDut, TimingEvent, TimingRun,
    TimingScheduler, TimingSchedulerError, TimingStage, TraceableDut, UnpackedArrayIndexError,
    extract_packed, extract_signed, extract_signed_packed, extract_unsigned, insert_packed,
    insert_signed, insert_signed_packed, insert_unsigned, unpacked_array_ordinal,
};
#[doc(inline)]
pub use vvm_macros::{Clock, Drive, Sample, test};

/// Commonly used VVM traits, derives, and testbench types.
///
/// Importing the prelude makes both the methodology traits and derive macros
/// available:
///
/// ```
/// use vvm::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        Bin, Bits, Clock, ClockScheduler, ClockTiming, CoverageGroup, CoverageGroupInstance,
        CoverageGroupVisitor, CoverageItemRef, Coverpoint, Cross2, CycleTiming, Drive, Dut,
        ExactScoreboard, FailurePolicy, InoutState, RandomContext, Randomize, ReferenceModel,
        ReplayableSequence, Sample, Scoreboard, Seed, SignedBits, SimulationTime, TestContext,
        TestDescriptor, TestRegistryError, TestRunConfig, Testbench, TimeStep, TimedDut,
        TimingScheduler,
    };
}

/// Includes a DUT wrapper generated by `vvm-build`.
///
/// The identifier must match the name passed to
/// `vvm_build::DutBuilder::new`.
///
/// The short form creates a private module:
///
/// ```ignore
/// vvm::include_dut!(counter);
///
/// use counter::Counter;
/// ```
///
/// An explicit module visibility may also be supplied:
///
/// ```ignore
/// vvm::include_dut!(pub mod counter);
/// ```
///
/// Multiple DUTs are naturally separated by module:
///
/// ```ignore
/// vvm::include_dut!(counter);
/// vvm::include_dut!(uart);
///
/// use counter::Counter;
/// use uart::Uart;
/// ```
///
/// Manual fallback:
///
/// ```ignore
/// mod counter {
///     include!(concat!(env!("OUT_DIR"), "/vvm/counter/generated/dut.rs"));
/// }
/// ```
#[macro_export]
macro_rules! include_dut {
    ($name:ident $(,)?) => {
        $crate::include_dut!(mod $name);
    };

    ($visibility:vis mod $name:ident $(,)?) => {
        #[doc(hidden)]
        extern crate vvm as cxx;

        #[doc = concat!(
            "Generated VVM DUT integration module for `",
            stringify!($name),
            "`."
        )]
        $visibility mod $name {
            include!(concat!(
                env!("OUT_DIR"),
                "/vvm/",
                stringify!($name),
                "/generated/dut.rs"
            ));
        }
    };
}

/// Implementation dependencies used by VVM-generated source.
///
/// This module is public only because generated source is compiled in the
/// consuming crate. It is not part of the user-facing API.
#[doc(hidden)]
pub mod __private {
    pub use cxx;

    pub use crate::test::{TestFailure, run_test};
}

#[doc(hidden)]
pub use cxx::*;

#[cfg(test)]
mod tests {
    use crate::prelude::{Bin, Coverpoint};

    #[test]
    fn coverage_primitives_are_usable_from_the_prelude() {
        let mut coverage = Coverpoint::builder("value")
            .bin(Bin::value("zero", 0_u8))
            .bin(Bin::inclusive_range("nonzero", 1_u8, u8::MAX))
            .build()
            .expect("valid coverpoint");
        let sample = coverage.sample(&0).expect("normal sample");

        assert!(sample.hit());
        assert_eq!(coverage.coverage().covered(), 1);
        assert_eq!(coverage.coverage().total(), 2);
    }
}
