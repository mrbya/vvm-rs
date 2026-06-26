use std::fmt;

use crate::{ReplayToken, SimulationTime, TestResult};

/// Classification of a registered test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestKind {
    /// Test with no randomized replay configuration.
    Deterministic,

    /// Test that accepts and reports a replay token.
    Replayable,
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Replayable => f.write_str("replayable"),
        }
    }
}

/// High-level outcome of a registered test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestStatus {
    /// Verification completed successfully.
    Passed,

    /// Verification completed with scoreboard failure.
    Failed,

    /// Setup, simulation or finalization failed.
    Error,
}

impl TestStatus {
    /// Returns the test passed.
    #[must_use]
    pub const fn passed(self) -> bool {
        matches!(self, Self::Passed)
    }
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Passed => f.write_str("PASS"),
            Self::Failed => f.write_str("FAIL"),
            Self::Error => f.write_str("ERROR"),
        }
    }
}

/// Configuration sipplied to one registerred test execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TestRunConfig {
    /// Explicit random-stream replay override.
    replay_token: Option<ReplayToken>,
}

impl TestRunConfig {
    /// Empty execution configuration.
    pub const EMPTY: Self = Self { replay_token: None };

    /// Creates an empty execution configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Overrides the test's default replay token.
    #[must_use]
    pub const fn with_replay_token(mut self, replay_token: ReplayToken) -> Self {
        self.replay_token = Some(replay_token);
        self
    }

    /// Returns configured replay token.
    #[must_use]
    pub const fn replay_token(&self) -> Option<ReplayToken> {
        self.replay_token
    }
}

/// Type-independent statistics from one completed testbench run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestStatistics {
    /// Completed cycles.
    cycles: u64,

    /// Performed scoreboard checks.
    checks: u64,

    /// Retained scoreboard check failures.
    check_failures: usize,

    /// Simulation start time.
    start_time: SimulationTime,

    /// Simulation end time.
    final_time: SimulationTime,

    /// Whether execution was stopped by failure policy.
    stopped_by_failure_policy: bool,
}

impl TestStatistics {
    /// Returns the number of completed cycles.
    #[must_use]
    pub const fn cycles(self) -> u64 {
        self.cycles
    }

    /// Returns the number of performed scoreboard checks.
    #[must_use]
    pub const fn checks(self) -> u64 {
        self.checks
    }

    /// Returns the number of retained scoreboard check failures.
    #[must_use]
    pub const fn check_failures(self) -> usize {
        self.check_failures
    }

    /// Returns the simulation time at which execution began.
    #[must_use]
    pub const fn start_time(self) -> SimulationTime {
        self.start_time
    }

    /// Returns the simulation time at which execution stopped.
    #[must_use]
    pub const fn final_time(self) -> SimulationTime {
        self.final_time
    }

    /// Returns whether execution was stopped by failure policy.
    #[must_use]
    pub const fn stopped_by_failure_policy(self) -> bool {
        self.stopped_by_failure_policy
    }
}

/// Type-erased result returned by a registered test function.
///
/// Individual tests remain strongly typed internally, Conversion to this type
/// happens only after the testbench has completed its run.
#[derive(Debug)]
pub struct TestOutcome {
    /// High-level execution status.
    status: TestStatus,

    /// Statistics from a completed testbench run.
    statistics: Option<TestStatistics>,

    /// Replay metadata, when available.
    replay_token: Option<ReplayToken>,

    /// Compact single-line report,
    summary: String,

    /// Detailed report.
    report: String,
}

impl TestOutcome {
    /// Converts a strongly typed testbench result into a registry outcome.
    #[must_use]
    pub fn from_result<S, F, E>(result: &TestResult<S, F, E>) -> Self
    where
        S: fmt::Debug,
        F: fmt::Display,
        E: fmt::Display,
    {
        let status = if result.passed() {
            TestStatus::Passed
        } else if result.simulation_error().is_some() || result.finalization_error().is_some() {
            TestStatus::Error
        } else {
            TestStatus::Failed
        };

        let statistics = TestStatistics {
            cycles: result.cycles(),
            checks: result.checks(),
            check_failures: result.failure_count(),
            start_time: result.start_time(),
            final_time: result.final_time(),
            stopped_by_failure_policy: result.stopped_by_failure_policy(),
        };

        let replay_token = result.replay_token();

        let summary = result.summary().to_string();

        let report = result.detailed_report().to_string();

        Self {
            status,
            statistics: Some(statistics),
            replay_token,
            summary,
            report,
        }
    }

    /// Creates an outcome for an error that occurred before or outside the
    /// testbench runner.
    #[must_use]
    pub fn error(error: impl fmt::Display) -> Self {
        let message = error.to_string();

        let summary = format!("ERROR: {message}");

        let report = format!("Test execution error:\n\n  error: {message}");

        Self {
            status: TestStatus::Error,
            statistics: None,
            replay_token: None,
            summary,
            report,
        }
    }

    /// Associates replay metadata with this outcome.
    #[must_use]
    pub const fn with_replay_token(mut self, replay_token: ReplayToken) -> Self {
        self.replay_token = Some(replay_token);
        self
    }

    /// Returns test status.
    #[must_use]
    pub const fn status(&self) -> TestStatus {
        self.status
    }

    /// Returns whether the test passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.status.passed()
    }

    /// Returns test statistics.
    #[must_use]
    pub const fn statistics(&self) -> Option<TestStatistics> {
        self.statistics
    }

    /// Returns replay token associated with test.
    #[must_use]
    pub const fn replay_token(&self) -> Option<ReplayToken> {
        self.replay_token
    }

    /// Returns test summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns test report.
    #[must_use]
    pub fn report(&self) -> &str {
        &self.report
    }
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.summary)
    }
}
