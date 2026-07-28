use std::num::NonZeroUsize;

/// Policy controlling when check failures stop a testbench run.
///
/// Everu policy has a finite maximum, preventing unbounded diagnostic storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailurePolicy {
    /// Maximum number of check failures retained before stopping.
    maximum_failures: NonZeroUsize,
}

impl FailurePolicy {
    /// Stops immediately after the first check failure.
    pub const STOP_ON_FIRST: Self = Self {
        maximum_failures: NonZeroUsize::MIN,
    };

    /// Creates a policy that collects at most `maximum_failures` failures.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied maximum is zero.
    pub const fn collect_up_to(maximum_failures: usize) -> Result<Self, InvalidFailureLimit> {
        match NonZeroUsize::new(maximum_failures) {
            Some(maximum_failures) => Ok(Self { maximum_failures }),
            None => Err(InvalidFailureLimit),
        }
    }

    /// Returns whether the current failure count requires termination.
    pub(crate) const fn should_stop(self, failure_count: usize) -> bool {
        failure_count >= self.maximum_failures.get()
    }
}

impl Default for FailurePolicy {
    fn default() -> Self {
        Self::STOP_ON_FIRST
    }
}

/// Error returned for a zero failure-retention limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
#[error("maximum failure count must be > 0")]
pub struct InvalidFailureLimit;

#[cfg(test)]
mod tests {
    use super::FailurePolicy;

    #[test]
    fn rejects_zero_and_stops_at_the_configured_limit() {
        let zero_limit = FailurePolicy::collect_up_to(0).expect_err("zero limit must be rejected");
        assert_eq!(zero_limit.to_string(), "maximum failure count must be > 0");

        let policy = FailurePolicy::collect_up_to(3).expect("nonzero limit must be accepted");

        assert!(!policy.should_stop(2));
        assert!(policy.should_stop(3));
        assert!(policy.should_stop(4));
        assert!(FailurePolicy::default().should_stop(1));
    }
}
