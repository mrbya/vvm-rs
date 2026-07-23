use crate::{
    CoverageGroup, CoverageSession, CoverageSessionError, CoverageSessionSnapshot, TestRunConfig,
};

/// Mutable execution context supplied to context-aware VVM tests.
///
/// It owns the resolved execution configuration and per-test coverage session.
pub struct TestContext {
    /// Resolved execution configuration.
    config: TestRunConfig,
    /// Per-test coverage session.
    coverage: CoverageSession,
}

impl TestContext {
    /// Creates an execution context for one resolved descriptor invocation.
    pub(crate) fn new(test_name: &'static str, config: TestRunConfig) -> Self {
        Self {
            config,
            coverage: CoverageSession::new_validated(test_name),
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

    /// Finalizes the owned per-test coverage session.
    pub(crate) fn finish(self) -> Option<CoverageSessionSnapshot> {
        self.coverage.finish()
    }
}
