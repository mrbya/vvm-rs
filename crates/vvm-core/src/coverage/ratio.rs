/// Exact normal-bin coverage ratio.
///
/// It holds exact integer counts. Only normal bins participate; ignore and
/// illegal bins are excluded. Percentage calculation is deliberately deferred
/// to reporting. [`Self::is_complete`] means every normal bin met its threshold.
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

#[cfg(test)]
mod tests {
    use super::CoverageRatio;

    #[test]
    fn accessors_return_exact_counts() {
        let ratio = CoverageRatio::new(2, 3, 5);

        assert_eq!(ratio.covered(), 2);
        assert_eq!(ratio.uncovered(), 3);
        assert_eq!(ratio.total(), 5);
        assert!(!ratio.is_complete());
    }

    #[test]
    fn complete_ratio_has_no_uncovered_bins() {
        assert!(CoverageRatio::new(2, 0, 2).is_complete());
    }
}
