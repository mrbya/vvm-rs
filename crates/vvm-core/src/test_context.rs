use thiserror::Error;

use crate::{
    CoverageGroup, CoverageRuntimeError, CoverageSession, CoverageSessionError,
    CoverageSessionSnapshot, TestRunConfig,
};

/// Category of framework diagnostic deferred through [`TestContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum TestDiagnostic {
    /// Functional-coverage sampling failed.
    #[error("coverage sampling: {source}")]
    CoverageSampling {
        /// Typed coverage runtime error.
        #[source]
        source: CoverageRuntimeError,
    },

    /// Functional-coverage snapshot capture failed.
    #[error("coverage capture: {source}")]
    CoverageCapture {
        /// Typed coverage session error.
        #[source]
        source: CoverageSessionError,
    },

    /// A coverage-capable test captured no groups.
    #[error(
        "missing coverage for test `{test}`: the test declared coverage but captured no groups"
    )]
    MissingCoverage {
        /// Registered test name.
        test: String,
    },
}

impl TestDiagnostic {
    /// Returns the framework diagnostic category.
    #[must_use]
    pub const fn kind(&self) -> TestDiagnosticKind {
        match *self {
            Self::CoverageSampling { .. } => TestDiagnosticKind::CoverageSampling,
            Self::CoverageCapture { .. } => TestDiagnosticKind::CoverageCapture,
            Self::MissingCoverage { .. } => TestDiagnosticKind::MissingCoverage,
        }
    }

    /// Returns the retained coverage runtime error when applicable.
    #[must_use]
    pub const fn coverage_runtime_error(&self) -> Option<&CoverageRuntimeError> {
        match *self {
            Self::CoverageSampling { ref source } => Some(source),
            Self::CoverageCapture { .. } | Self::MissingCoverage { .. } => None,
        }
    }

    /// Returns the retained coverage session error when applicable.
    #[must_use]
    pub const fn coverage_session_error(&self) -> Option<&CoverageSessionError> {
        match *self {
            Self::CoverageCapture { ref source } => Some(source),
            Self::CoverageSampling { .. } | Self::MissingCoverage { .. } => None,
        }
    }

    /// Returns the test name when the diagnostic concerns missing coverage.
    #[must_use]
    pub fn test_name(&self) -> Option<&str> {
        match *self {
            Self::MissingCoverage { ref test } => Some(test),
            Self::CoverageSampling { .. } | Self::CoverageCapture { .. } => None,
        }
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
    pub(crate) fn record_coverage_sampling_error(&mut self, error: CoverageRuntimeError) {
        self.diagnostics
            .push(TestDiagnostic::CoverageSampling { source: error });
    }

    /// Records one deferred coverage capture failure.
    pub(crate) fn record_coverage_capture_error(&mut self, error: CoverageSessionError) {
        self.diagnostics
            .push(TestDiagnostic::CoverageCapture { source: error });
    }

    /// Finalizes the owned per-test coverage session.
    pub(crate) fn finish(self) -> FinishedTestContext {
        FinishedTestContext {
            coverage: self.coverage.finish(),
            diagnostics: self.diagnostics.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use crate::{
        CoverageCounterKind, CoverageRuntimeError, CoverageSampleError, SimulationTime,
        TestDiagnostic, TestDiagnosticKind,
    };

    #[test]
    fn sampling_diagnostic_retains_typed_source() {
        let runtime = CoverageRuntimeError::Coverpoint {
            item: "opcode",
            cycle: 7,
            time: SimulationTime::from_ticks(12),
            source: CoverageSampleError::CounterOverflow {
                coverpoint: "opcode".to_owned(),
                bin: Some("reserved".to_owned()),
                counter: CoverageCounterKind::BinHits,
            },
        };
        let diagnostic = TestDiagnostic::CoverageSampling { source: runtime };

        assert_eq!(diagnostic.kind(), TestDiagnosticKind::CoverageSampling);
        assert_eq!(
            diagnostic
                .coverage_runtime_error()
                .map(CoverageRuntimeError::item),
            Some("opcode")
        );
        assert!(diagnostic.coverage_session_error().is_none());
        assert!(diagnostic.test_name().is_none());
        assert!(
            diagnostic
                .source()
                .is_some_and(<dyn std::error::Error>::is::<CoverageRuntimeError>)
        );
        assert_eq!(
            diagnostic.to_string(),
            "coverage sampling: coverage coverpoint `opcode` failed at cycle 7 at 12 ticks: \
             functional coverage counter overflow in `opcode.reserved`: bin hit count",
        );
    }
}
