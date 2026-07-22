/// Exact normal-bin coverage ratio.
///
/// Only normal bins contribute to this metric. Ignores and illegal
/// bins are excluded from both the numerator and denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoverageRatio {
    /// Normal bins meeting their required hit count.
    covered: usize,

    /// Normal bins below their required hit count.
    uncovered: usize,

    /// Total number of normal bins.
    total: usize,
}

impl CoverageRatio {
    /// Creates one exact coverage ratio.
    pub(crate) const fn new(covered: usize, uncovered: usize, total: usize) -> Self {
        Self {
            covered,
            uncovered,
            total,
        }
    }

    /// Returns the number of covered normal bins.
    #[must_use]
    pub const fn covered(self) -> usize {
        self.covered
    }

    /// Returns the number of uncovered normal bins.
    #[must_use]
    pub const fn uncovered(self) -> usize {
        self.uncovered
    }

    /// Returns the total number of normal bins.
    #[must_use]
    pub const fn total(self) -> usize {
        self.total
    }

    /// Returns whether every normal bin is covered.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.uncovered == 0
    }
}
