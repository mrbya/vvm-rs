//! Cycle-driven testbench execution, reference models, and scoreboards.

pub use vvm_core::{
    CheckFailure, ExactScoreboard, FailurePolicy, InvalidFailureLimit as FailureLimitError,
    Mismatch, ObservedCycle, ReferenceModel, Scoreboard, SimulationError, SimulationStage,
    TestResult, Testbench,
};
