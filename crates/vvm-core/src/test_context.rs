use crate::{
    CoverageGroup, CoverageRuntimeError, CoverageSession, CoverageSessionError,
    CoverageSessionSnapshot, TestRunConfig,
};

/// Category of framework diagnostic deferred through [`TestContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestDiagnosticKind {
    /// Functional-coverage sampling failed.
    CoverageSampling,
    /// Functional-coverage snapshot capture failed.
    CoverageCapture,
    /// A declared coverage-capable test captured no coverage.
    MissingCoverage,
}

impl std::fmt::Display for TestDiagnosticKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::CoverageSampling => formatter.write_str("coverage sampling"),
            Self::CoverageCapture => formatter.write_str("coverage capture"),
            Self::MissingCoverage => formatter.write_str("missing coverage"),
        }
    }
}

/// One deferred framework diagnostic retained with the test outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDiagnostic {
    /// Diagnostic category.
    kind: TestDiagnosticKind,
    /// Human-readable deterministic description.
    message: String,
}

impl TestDiagnostic {
    /// Returns the framework diagnostic category.
    #[must_use]
    pub const fn kind(&self) -> TestDiagnosticKind {
        self.kind
    }

    /// Returns the deterministic diagnostic description.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Creates one framework diagnostic.
    pub(crate) const fn new(kind: TestDiagnosticKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl std::fmt::Display for TestDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.kind, self.message)
    }
}

/// Finalized per-test context state.
pub struct FinishedTestContext {
    /// Frozen coverage session, when groups were captured.
    pub(crate) coverage: Option<CoverageSessionSnapshot>,
    /// Deferred framework diagnostics in recording order.
    pub(crate) diagnostics: Box<[TestDiagnostic]>,
}

/// Mutable execution context supplied to context-aware VVM tests.
///
/// It owns the resolved execution configuration and per-test coverage session.
pub struct TestContext {
    /// Resolved execution configuration.
    config: TestRunConfig,
    /// Per-test coverage session.
    coverage: CoverageSession,
    /// Deferred framework diagnostics.
    diagnostics: Vec<TestDiagnostic>,
}

impl TestContext {
    /// Creates an execution context for one resolved descriptor invocation.
    pub(crate) fn new(test_name: &'static str, config: TestRunConfig) -> Self {
        Self {
            config,
            coverage: CoverageSession::new_validated(test_name),
            diagnostics: Vec::new(),
        }
    }

    /// Returns the stable test name.
    #[must_use]
    pub fn test_name(&self) -> &str {
        self.coverage.test_name()
    }

    /// Returns the resolved execution configuration.
    #[must_use]
    pub const fn config(&self) -> &TestRunConfig {
        &self.config
    }

    /// Captures one coverage-group instance at its current state.
    ///
    /// Capture freezes the group at this call; later sampling does not update
    /// the captured snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error reported by the per-test coverage session.
    pub fn capture_coverage<G>(&mut self, group: &G) -> Result<(), CoverageSessionError>
    where
        G: CoverageGroup + ?Sized,
    {
        self.coverage.capture(group)
    }

    /// Returns the current session for read-only inspection.
    #[must_use]
    pub const fn coverage_session(&self) -> &CoverageSession {
        &self.coverage
    }

    /// Returns framework diagnostics recorded during test execution.
    #[must_use]
    pub fn diagnostics(&self) -> &[TestDiagnostic] {
        &self.diagnostics
    }

    /// Records one deferred coverage sampling failure.
    pub(crate) fn record_coverage_sampling_error(&mut self, error: &CoverageRuntimeError) {
        self.diagnostics.push(TestDiagnostic::new(
            TestDiagnosticKind::CoverageSampling,
            error.to_string(),
        ));
    }

    /// Records one deferred coverage capture failure.
    pub(crate) fn record_coverage_capture_error(&mut self, error: &CoverageSessionError) {
        self.diagnostics.push(TestDiagnostic::new(
            TestDiagnosticKind::CoverageCapture,
            error.to_string(),
        ));
    }

    /// Finalizes the owned per-test coverage session.
    pub(crate) fn finish(self) -> FinishedTestContext {
        FinishedTestContext {
            coverage: self.coverage.finish(),
            diagnostics: self.diagnostics.into_boxed_slice(),
        }
    }
}
