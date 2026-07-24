use core::fmt;

use crate::CoverageRatio;

/// Stable reporting representation of one coverage percentage.
///
/// The value is stored in basis points. Ten thousand basis points represent
/// exactly 100.00 percent.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoveragePercentage {
    /// Hundredths of one percent.
    basis_points: u16,
}

impl CoveragePercentage {
    /// Converts one exact coverage ratio into a fixed-point presentation percentage.
    #[must_use]
    pub fn from_ratio(ratio: CoverageRatio) -> Self {
        const FULL_SCALE: u128 = 10_000;
        const INCOMPLETE_MAXIMUM: u128 = 9_999;

        let total = ratio.total();
        if total == 0 {
            return Self::default();
        }

        let numerator = u128::try_from(ratio.covered()).map_or(u128::MAX, |value| value);
        let denominator = u128::try_from(total).map_or(u128::MAX, |value| value);
        let scaled = numerator.saturating_mul(FULL_SCALE);
        let half = denominator.checked_div(2).unwrap_or_default();
        let rounded = scaled
            .saturating_add(half)
            .checked_div(denominator)
            .unwrap_or_default();
        let capped = if ratio.is_complete() {
            rounded.min(FULL_SCALE)
        } else {
            rounded.min(INCOMPLETE_MAXIMUM)
        };
        let basis_points = u16::try_from(capped).map_or(u16::MAX, |value| value);

        Self { basis_points }
    }

    /// Returns the value in basis points.
    #[must_use]
    pub const fn basis_points(self) -> u16 {
        self.basis_points
    }

    /// Returns the whole percentage component.
    #[must_use]
    pub const fn whole_percent(self) -> u8 {
        let [whole, _] = self.basis_points.div_euclid(100).to_le_bytes();
        whole
    }

    /// Returns the hundredths component.
    #[must_use]
    pub const fn fractional_hundredths(self) -> u8 {
        let [fractional, _] = self.basis_points.rem_euclid(100).to_le_bytes();
        fractional
    }
}

impl fmt::Display for CoveragePercentage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{:02}%",
            self.whole_percent(),
            self.fractional_hundredths()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CoveragePercentage;
    use crate::CoverageRatio;

    fn percentage(covered: usize, uncovered: usize) -> CoveragePercentage {
        CoveragePercentage::from_ratio(CoverageRatio::new(
            covered,
            uncovered,
            covered.saturating_add(uncovered),
        ))
    }

    #[test]
    fn formats_expected_fixed_point_values() {
        assert_eq!(percentage(0, 4).to_string(), "0.00%");
        assert_eq!(percentage(1, 3).to_string(), "25.00%");
        assert_eq!(percentage(1, 2).to_string(), "33.33%");
        assert_eq!(percentage(2, 1).to_string(), "66.67%");
        assert_eq!(percentage(1, 7).to_string(), "12.50%");
        assert_eq!(percentage(8, 0).to_string(), "100.00%");
    }

    #[test]
    fn rounds_half_up_and_clamps_incomplete_ratios() {
        assert_eq!(percentage(1, 1_599).to_string(), "0.06%");
        assert_eq!(percentage(999_999, 1).to_string(), "99.99%");
    }

    #[test]
    fn zero_denominator_is_zero() {
        assert_eq!(
            CoveragePercentage::from_ratio(CoverageRatio::new(0, 0, 0)).to_string(),
            "0.00%"
        );
    }

    #[test]
    fn accessors_are_exact() {
        let percentage = percentage(7, 1);
        assert_eq!(percentage.basis_points(), 8_750);
        assert_eq!(percentage.whole_percent(), 87);
        assert_eq!(percentage.fractional_hundredths(), 50);
    }
}
