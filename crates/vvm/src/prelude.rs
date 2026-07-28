//! Common imports for ordinary cycle-driven test authors.
//!
//! The prelude intentionally includes derives, DUT traits, basic testbench
//! components, and test configuration only. Advanced coverage, packed values,
//! schedulers, and registry APIs require explicit domain imports.

pub use crate::dut::{Drive, Dut, Sample};
pub use crate::test::{TestContext, TestRunConfig};
pub use crate::testbench::{
    ExactScoreboard, FailurePolicy, Mismatch, ReferenceModel, Scoreboard, TestResult, Testbench,
};
pub use crate::timing::Clock;
pub use crate::{Clock, Coverage, Drive, Sample};
