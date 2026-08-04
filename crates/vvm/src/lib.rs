//! Verilator Verification Methodology for Rust.
//!
//! This is the recommended public entry point for VVM verification code. Use
//! [`include_dut!`] to include a wrapper produced by `vvm-build`, then compose
//! the modules below into ordinary Rust tests. The task-oriented guide is at
//! <https://byacrates.gitlab.io/vvm-rs/> and the maintained examples live in
//! the repository `examples/` directory.
//!
//! VVM's public API is organized by domain. Import ordinary types from the
//! relevant facade module, such as [`dut`], [`testbench`], or [`coverage`].
//! Derive and attribute macros remain available at the crate root.
//!
//! ```
//! use vvm::coverage::{Bin, Coverpoint, Cross2};
//!
//! let mut value = Coverpoint::builder("value")
//!     .bin(Bin::value("zero", 0_u8))
//!     .build()?;
//! let mut response = Coverpoint::builder("response")
//!     .bin(Bin::value("okay", true))
//!     .build()?;
//! let mut cross = Cross2::builder("value_x_response", &value, &response).build()?;
//! let value_sample = value.sample(&0)?;
//! let response_sample = response.sample(&true)?;
//! assert!(cross.sample(&value_sample, &response_sample)?.hit());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Standard Rust test-harness bridge used by generated test wrappers.
mod harness;

/// Functional coverage definitions, persistence, reports, and sessions.
pub mod coverage;
/// DUT lifecycle, driving, sampling, and tracing traits.
pub mod dut;
/// Packed bit vectors and packed-layout utilities.
pub mod packed;
/// Common test-authoring imports.
pub mod prelude;
/// Deterministic randomization and replay support.
pub mod random;
/// Registered test execution and reporting APIs.
pub mod test;
/// Cycle-driven testbench, models, and scoreboards.
pub mod testbench;
/// Clocks, simulation time, and timing schedulers.
pub mod timing;

/// Derives [`timing::Clock`] for a unit clock-driver type.
///
/// Use this derive for cycle-driven tests that want the default scheduler-owned
/// clock driver rather than a handwritten clock type.
///
/// Required container attribute:
///
/// - `#[vvm(dut = path::ToDut, clock = "clk_name")]`
///
/// Supported helper attributes:
///
/// - `edge = "rising"` for the default active edge;
/// - `edge = "falling"` for a falling-edge clock.
///
/// The target type is typically a fieldless struct because the derive generates
/// behavior, not captured runtime state.
///
/// # Example
///
/// ```text
/// #[derive(vvm::Clock)]
/// #[vvm(dut = crate::counter::Counter, clock = "clk")]
/// struct CounterClock;
/// ```
pub use vvm_macros::Clock;
/// Derives a typed functional-coverage model.
///
/// Use this derive when one reusable design or subsystem has a stable coverage
/// definition built from coverpoints and crosses.
///
/// Required container attribute:
///
/// - `#[vvm(definition = "name", revision = N, stimulus = StimulusType, observation = ObservationType)]`
///
/// Supported field attributes:
///
/// - `#[vvm(coverpoint(build = builder_fn, sample = sample_fn))]`
/// - `#[vvm(cross(left = field_name, right = other_field_name))]`
///
/// The derive generates the coverage-definition, visitation, and runtime
/// sampling glue. Use manual coverage construction when the model is too dynamic
/// for a static struct definition.
///
/// # Example
///
/// ```text
/// use vvm::coverage::{Bin, BuildError, Coverpoint, Cross2};
/// use vvm::testbench::ObservedCycle;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Activity {
///     Idle,
///     Fire,
/// }
///
/// #[derive(vvm::Coverage)]
/// #[vvm(
///     definition = "controller_coverage",
///     revision = 1,
///     stimulus = crate::Stimulus,
///     observation = crate::Observation,
/// )]
/// struct ControllerCoverage {
///     #[vvm(coverpoint(build = activity_coverpoint, sample = activity_for))]
///     activity: Coverpoint<Activity>,
///     #[vvm(coverpoint(build = ready_coverpoint, sample = ready_for))]
///     ready: Coverpoint<bool>,
///     #[vvm(cross(left = activity, right = ready))]
///     activity_x_ready: Cross2,
/// }
///
/// # fn activity_coverpoint(name: &'static str) -> Result<Coverpoint<Activity>, BuildError> {
/// #     Coverpoint::builder(name).bin(Bin::value("idle", Activity::Idle)).bin(Bin::value("fire", Activity::Fire)).build()
/// # }
/// # fn ready_coverpoint(name: &'static str) -> Result<Coverpoint<bool>, BuildError> {
/// #     Coverpoint::builder(name).bin(Bin::value("ready", true)).bin(Bin::value("stall", false)).build()
/// # }
/// # fn activity_for(_: ObservedCycle<'_, crate::Stimulus, crate::Observation>) -> Activity { Activity::Idle }
/// # fn ready_for(_: ObservedCycle<'_, crate::Stimulus, crate::Observation>) -> bool { true }
/// ```
pub use vvm_macros::Coverage;
/// Derives [`dut::Drive`] for a named-field stimulus structure.
///
/// Use `Drive` for values that write DUT inputs. The derive generates one
/// [`dut::Drive`] implementation for the annotated type and maps each marked
/// field to the corresponding generated DUT setter.
///
/// Required container attribute:
///
/// - `#[vvm(dut = path::ToDut)]`
///
/// Supported field attributes:
///
/// - `#[vvm(port)]` to map a field to a same-named generated DUT port;
/// - `#[vvm(port = "dut_port_name")]` to map to an explicit generated port.
///
/// Supported input shape:
///
/// - a struct with named fields;
/// - field types that match the generated DUT setter types.
///
/// Unsupported shapes produce compile-time diagnostics, including tuple structs,
/// enums, missing `dut` attributes, and unmapped fields.
///
/// # Example
///
/// ```text
/// #[derive(vvm::Drive)]
/// #[vvm(dut = crate::counter::Counter)]
/// struct Stimulus {
///     #[vvm(port)]
///     reset_n: bool,
///     #[vvm(port = "enable_i")]
///     enable: bool,
/// }
/// ```
pub use vvm_macros::Drive;
/// Derives [`dut::Sample`] for a named-field observation structure.
///
/// Use `Sample` for values that read DUT outputs after evaluation. The derive
/// generates one [`dut::Sample`] implementation and maps each marked field to a
/// generated DUT getter.
///
/// Required container attribute:
///
/// - `#[vvm(dut = path::ToDut)]`
///
/// Supported field attributes:
///
/// - `#[vvm(port)]` to read a same-named generated DUT port;
/// - `#[vvm(port = "dut_port_name")]` to read an explicitly named port.
///
/// Every sampled field must be mapped explicitly. Use a manual
/// [`dut::Sample`] implementation when a semantic observation type is clearer
/// than a direct field-to-port mirror.
///
/// # Example
///
/// ```text
/// #[derive(vvm::Sample)]
/// #[vvm(dut = crate::counter::Counter)]
/// struct Observation {
///     #[vvm(port)]
///     count: u8,
/// }
/// ```
pub use vvm_macros::Sample;
/// Registers a VVM test while preserving normal Rust test discovery.
///
/// `#[vvm::test]` expands to a visible Rust `#[test]` wrapper plus hidden VVM
/// descriptor and adapter items. The original function still defines the test's
/// verification logic; the macro adds registration, configuration handling, and
/// optional coverage/tracing capabilities around it.
///
/// Supported function parameters:
///
/// - no arguments;
/// - `&vvm::test::TestRunConfig` for configuration-only tests;
/// - `&mut vvm::test::TestContext` for context-aware tests that need coverage or
///   retained framework diagnostics.
///
/// Supported return forms include plain results and testbench result wrappers
/// accepted by VVM's outcome conversion layer.
///
/// Supported capability attributes:
///
/// - `trace`
/// - `coverage`
/// - `cycles`
/// - `replay(default = CONST_REPLAY_TOKEN)`
///
/// Important behavior:
///
/// - Cargo and nextest still discover the generated Rust test normally;
/// - ignored tests remain ordinary Rust `#[ignore]` tests;
/// - test filtering still works through the generated visible test name;
/// - environment variables such as `VVM_REPLAY`, `VVM_SEED`, `VVM_CYCLES`,
///   `VVM_TRACE_DIR`, and `VVM_COVERAGE_DIR` affect only the capabilities the
///   test actually declared.
///
/// Use `TestRunConfig` when the test only needs configuration values. Use
/// `TestContext` when the test needs coverage capture or retained framework
/// diagnostics.
///
/// # Minimal Example
///
/// ```no_run
/// use std::convert::Infallible;
///
/// use vvm::testbench::TestResult;
/// use vvm::timing::SimulationTime;
///
/// /// Minimal deterministic smoke test.
/// #[vvm::test]
/// fn smoke() -> Result<TestResult<(), String, Infallible>, Infallible> {
///     Ok(TestResult::completed(
///         SimulationTime::ZERO,
///         SimulationTime::ZERO,
///         None,
///     ))
/// }
/// ```
///
/// # Configured Example
///
/// ```no_run
/// use std::convert::Infallible;
///
/// use vvm::test::TestContext;
/// use vvm::testbench::TestResult;
/// use vvm::timing::SimulationTime;
///
/// /// Covered smoke test with retained runtime context.
/// #[vvm::test(trace, coverage, cycles)]
/// fn covered_smoke(
///     context: &mut TestContext,
/// ) -> Result<TestResult<(), String, Infallible>, Infallible> {
///     let _config = context.config();
///     Ok(TestResult::completed(
///         SimulationTime::ZERO,
///         SimulationTime::ZERO,
///         None,
///     ))
/// }
/// ```
pub use vvm_macros::test;

/// Includes a DUT wrapper generated by `vvm-build`.
///
/// The logical DUT name must match the name passed to
/// `vvm_build::DutBuilder::new(...)`. The macro expands to a Rust module containing
/// the generated DUT type, associated generated error type, and generated port
/// accessors.
///
/// Supported forms:
///
/// - `vvm::include_dut!(counter);`
/// - `vvm::include_dut!(pub mod counter);`
///
/// Use the second form when the generated module must be re-exported from a
/// library crate.
///
/// Common failures:
///
/// - the build script never called `DutBuilder::build()`;
/// - the logical name does not match between `DutBuilder::new("name")` and
///   `include_dut!(name)`;
/// - the macro is expanded before Cargo has produced the generated source.
///
/// # Example
///
/// ```text
/// // build.rs
/// # fn build() -> Result<(), Box<dyn std::error::Error>> {
/// vvm_build::DutBuilder::new("counter")
///     .top_module("counter")
///     .source("rtl/counter.sv")
///     .build()?;
/// # Ok(()) }
///
/// // src/lib.rs or src/main.rs
/// vvm::include_dut!(counter);
/// ```
#[macro_export]
macro_rules! include_dut {
    ($name:ident $(,)?) => { $crate::include_dut!(mod $name); };
    ($visibility:vis mod $name:ident $(,)?) => {
        #[doc(hidden)]
        extern crate vvm as cxx;

        #[doc = concat!("Generated VVM DUT integration module for `", stringify!($name), "`.")]
        $visibility mod $name {
            include!(concat!(env!("OUT_DIR"), "/vvm/", stringify!($name), "/generated/dut.rs"));
        }
    };
}

/// Generated-source implementation ABI.
///
/// This is public solely because VVM-generated Rust source is compiled inside
/// consuming crates. It is not a supported user-facing API; users must not
/// import it, and its layout may change with generated-code implementations.
#[doc(hidden)]
pub mod __private {
    pub use cxx;
    pub use vvm_core::{
        Bits, Clock, CoverageDefinitionError, CoverageGroupVisitor, CoverageInstance,
        CoverageItemRef, CoverageRuntimeError, CoverageSampleSpec, CoverageSpec, Cross2, Drive,
        Dut, InoutState, IntoTestOutcome, InvalidBitVectorWordCount, ObservedCycle,
        PackedEnumLayout, PackedEnumVariantLayout, PackedFieldLayout, PackedLayout,
        PackedLayoutError, PackedRange, PackedValue, Sample, SignedBits, SimulationTime,
        TestCapabilities, TestContext, TestDescriptor, TestOutcome, TimeStep, TimedDut,
        TraceableDut, UnpackedArrayIndexError, extract_packed, extract_signed,
        extract_signed_packed, extract_unsigned, insert_packed, insert_signed,
        insert_signed_packed, insert_unsigned, unpacked_array_ordinal,
    };

    pub use crate::harness::{TestFailure, run_test};
}

#[doc(hidden)]
pub use cxx::*;
