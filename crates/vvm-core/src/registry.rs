use std::fmt;

use crate::{ReplayToken, SimulationTime, TestResult};

/// Function implementing one registered test.
pub type TestFunction = fn(&TestRunConfig) -> TestOutcome;

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

    /// Defaults to a provided replay token if configured without replay.
    #[must_use]
    pub const fn replay_token_or(&self, default: ReplayToken) -> ReplayToken {
        match self.replay_token {
            Some(replay) => replay,
            None => default,
        }
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

/// Translates provided error/return type to a test outcome.
pub trait IntoTestOutcome {
    /// Converts provided type into test outcome..
    fn into_test_outcome(self) -> TestOutcome;

    /// Coverts provided type into test outcome with test replay context.
    fn into_test_outcome_with_replay(self, replay: ReplayToken) -> TestOutcome;
}

/// Metadata and entry point for one registered test.
#[derive(Debug, Clone, Copy)]
pub struct TestDescriptor {
    /// Stable registry name.
    name: &'static str,

    /// Human-readable description.
    description: &'static str,

    /// Test classification.
    kind: TestKind,

    /// Execution entry point.
    function: TestFunction,
}

impl TestDescriptor {
    /// Creates a deterministic test descriptor.
    #[must_use]
    pub const fn deterministic(
        name: &'static str,
        description: &'static str,
        function: TestFunction,
    ) -> Self {
        Self {
            name,
            description,
            kind: TestKind::Deterministic,
            function,
        }
    }

    /// Creates a replayable test descriptor.
    #[must_use]
    pub const fn replayable(
        name: &'static str,
        description: &'static str,
        function: TestFunction,
    ) -> Self {
        Self {
            name,
            description,
            kind: TestKind::Replayable,
            function,
        }
    }

    /// Returns test name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns test description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    /// Returns test kind.
    #[must_use]
    pub const fn kind(&self) -> TestKind {
        self.kind
    }
}

/// Error returned when constructing or querying a test registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestRegistryError {
    /// A descriptor has an invalid test name.
    InvalidName {
        /// Invalid name.
        name: &'static str,
    },

    /// Two descriptors have the same name.
    DuplicateName {
        /// Duplicated name.
        name: &'static str,
    },

    /// The requested test does not exist.
    UnknownTest {
        /// Requested test name.
        name: String,
    },

    /// Replay configuration was supplied to a deterministic test.
    ReplayNotSupported {
        /// Deterministic test name.
        name: &'static str,
    },
}

impl fmt::Display for TestRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidName { name } => write!(f, "invalid registered test name `{name}`"),
            Self::DuplicateName { name } => write!(f, "duplicate registered test name `{name}`"),
            Self::UnknownTest { ref name } => write!(f, "unknown registered test {name}"),
            Self::ReplayNotSupported { name } => {
                write!(f, "test `{name}` does not accept replay configuration")
            }
        }
    }
}

impl std::error::Error for TestRegistryError {}

/// Validated, ordered collection of registered tests.
#[derive(Debug, Clone, Copy)]
pub struct TestRegistry<'a> {
    /// Tests in declaration order.
    tests: &'a [TestDescriptor],
}

impl<'a> TestRegistry<'a> {
    /// Creates and validates a test registry.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid or duplicate names.
    pub fn new(tests: &'a [TestDescriptor]) -> Result<Self, TestRegistryError> {
        for (index, test) in tests.iter().enumerate() {
            validate_test_name(test.name())?;

            let duplicate = tests
                .iter()
                .take(index)
                .any(|candidate| candidate.name() == test.name());

            if duplicate {
                return Err(TestRegistryError::DuplicateName { name: test.name() });
            }
        }

        Ok(Self { tests })
    }

    /// Returns registered tests.
    #[must_use]
    pub const fn tests(&self) -> &'a [TestDescriptor] {
        self.tests
    }

    /// Returns number of registered tests.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.tests.len()
    }

    /// Returns whether the registry is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.tests.is_empty()
    }

    /// Finds test in registry.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&'a TestDescriptor> {
        self.tests.iter().find(|test| test.name() == name)
    }

    /// Executes a registered test by exact name.
    ///
    /// # Errors
    ///
    /// Returns an error if the test is unknown or the supplied configuration
    /// is incompatible with the descriptor.
    pub fn run(
        &self,
        name: &str,
        config: &TestRunConfig,
    ) -> Result<TestRun<'a>, TestRegistryError> {
        let test = self
            .find(name)
            .ok_or_else(|| TestRegistryError::UnknownTest {
                name: name.to_owned(),
            })?;

        if config.replay_token().is_some() && test.kind() == TestKind::Deterministic {
            return Err(TestRegistryError::ReplayNotSupported { name: test.name() });
        }

        let outcome = (test.function)(config);

        Ok(TestRun { test, outcome })
    }
}

/// Validates test name.
fn validate_test_name(name: &'static str) -> Result<(), TestRegistryError> {
    let mut characters = name.chars();

    let Some(first) = characters.next() else {
        return Err(TestRegistryError::InvalidName { name });
    };

    if first.is_ascii_lowercase()
        && characters.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_' | '.')
        })
    {
        return Ok(());
    }

    Err(TestRegistryError::InvalidName { name })
}

/// Completed execution of one registered test.
#[derive(Debug)]
pub struct TestRun<'a> {
    /// Executed descriptor.
    test: &'a TestDescriptor,

    /// Type-erased execution outcome.
    outcome: TestOutcome,
}

impl<'a> TestRun<'a> {
    /// Returns test descriptor.
    #[must_use]
    pub const fn test(&self) -> &'a TestDescriptor {
        self.test
    }

    /// Returns test status.
    #[must_use]
    pub const fn status(&self) -> TestStatus {
        self.outcome.status()
    }

    /// Returns whether the test passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.outcome.passed()
    }

    /// Returns test outcome.
    #[must_use]
    pub const fn outcome(&self) -> &TestOutcome {
        &self.outcome
    }

    /// Consumes test run.
    #[must_use]
    pub fn into_outcome(self) -> TestOutcome {
        self.outcome
    }
}

impl fmt::Display for TestRun<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}",
            self.test.name(),
            self.outcome.summary(),
        )
    }
}
