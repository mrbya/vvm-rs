use vvm::coverage::{Bin, CoverageBuildError, Coverpoint, Cross2};
use vvm::testbench::ObservedCycle;

use crate::verification::{CounterObservation, CounterStimulus};

/// Semantic operation active during one sampled counter cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CounterOperation {
    /// Reset is asserted.
    Reset,
    /// Reset is released and counting is disabled.
    Hold,
    /// Reset is released and counting is enabled.
    Count,
    /// Reserved invalid semantic value.
    Invalid,
}

/// Functional coverage for one counter DUT instance.
#[derive(vvm::Coverage)]
#[vvm(definition = "counter_coverage", revision = 1, stimulus = CounterStimulus, observation = CounterObservation)]
pub struct CounterCoverage {
    /// Reset, hold, and count operation.
    #[vvm(coverpoint(build = operation_coverpoint, sample = operation_for))]
    operation: Coverpoint<CounterOperation>,
    /// Sampled counter output region.
    #[vvm(coverpoint(build = count_region_coverpoint, sample = count_for))]
    count_region: Coverpoint<u8>,
    /// Sampled output parity, where `false` is even and `true` is odd.
    #[vvm(coverpoint(build = parity_coverpoint, sample = parity_for))]
    parity: Coverpoint<bool>,
    /// Operation against output parity.
    #[vvm(cross(left = operation, right = parity))]
    operation_x_parity: Cross2,
    /// Operation against output region.
    #[vvm(cross(left = operation, right = count_region))]
    operation_x_count_region: Cross2,
}

/// Builds the operation coverpoint.
fn operation_coverpoint(
    name: &'static str,
) -> Result<Coverpoint<CounterOperation>, CoverageBuildError> {
    Coverpoint::builder(name)
        .bin(Bin::value("hold", CounterOperation::Hold))
        .bin(Bin::value("count", CounterOperation::Count))
        .ignore_bin(Bin::value("reset", CounterOperation::Reset))
        .illegal_bin(Bin::value("invalid", CounterOperation::Invalid))
        .build()
}

/// Builds the count-region coverpoint.
fn count_region_coverpoint(name: &'static str) -> Result<Coverpoint<u8>, CoverageBuildError> {
    Coverpoint::builder(name)
        .bin(Bin::value("zero", 0_u8))
        .bin(Bin::inclusive_range("low", 1_u8, 15_u8))
        .bin(Bin::inclusive_range("medium", 16_u8, 63_u8))
        .bin(Bin::inclusive_range("high", 64_u8, 254_u8))
        .bin(Bin::value("maximum", u8::MAX))
        .build()
}

/// Builds the output parity coverpoint.
fn parity_coverpoint(name: &'static str) -> Result<Coverpoint<bool>, CoverageBuildError> {
    Coverpoint::builder(name)
        .bin(Bin::value("even", false))
        .bin(Bin::value("odd", true))
        .build()
}

/// Derives the operation represented by valid counter stimulus.
const fn operation_for(
    cycle: ObservedCycle<'_, CounterStimulus, CounterObservation>,
) -> CounterOperation {
    if !cycle.stimulus().reset_n() {
        CounterOperation::Reset
    } else if cycle.stimulus().enable() {
        CounterOperation::Count
    } else {
        CounterOperation::Hold
    }
}

/// Returns the observed count for coverage sampling.
const fn count_for(cycle: ObservedCycle<'_, CounterStimulus, CounterObservation>) -> u8 {
    cycle.observed().count()
}

/// Returns observed count parity for coverage sampling.
const fn parity_for(cycle: ObservedCycle<'_, CounterStimulus, CounterObservation>) -> bool {
    count_for(cycle) & 1 != 0
}

#[cfg(test)]
mod tests {
    use vvm::coverage::{CoverageGroup, CoverageGroupVisitor, CoverageItemRef};

    use super::CounterCoverage;

    #[test]
    fn counter_coverage_builds_and_validates() -> Result<(), Box<dyn std::error::Error>> {
        let coverage = CounterCoverage::new("dut.counter")?;

        coverage.validate()?;
        assert_eq!(coverage.instance().definition_name(), "counter_coverage");
        assert_eq!(coverage.instance().definition_revision(), 1);
        assert_eq!(coverage.instance().instance_path(), "dut.counter");

        Ok(())
    }

    #[test]
    fn group_visitation_order_is_stable() -> Result<(), Box<dyn std::error::Error>> {
        struct Names(Vec<String>);
        impl CoverageGroupVisitor for Names {
            fn visit(&mut self, item: CoverageItemRef<'_>) {
                self.0.push(item.name().to_owned());
            }
        }

        let coverage = CounterCoverage::new("dut.counter")?;
        let mut names = Names(Vec::new());
        coverage.visit_items(&mut names);

        assert_eq!(
            names.0,
            [
                "operation",
                "count_region",
                "parity",
                "operation_x_parity",
                "operation_x_count_region"
            ]
        );
        assert_eq!(coverage.summary()?.coverpoint_count(), 3);
        assert_eq!(coverage.summary()?.cross_count(), 2);

        Ok(())
    }
}
