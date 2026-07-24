use thiserror::Error;
use vvm::{
    Bin, CoverageBuildError, CoverageGroup, CoverageGroupError, CoverageGroupInstance,
    CoverageGroupVisitor, CoverageItemRef, CoverageSampleError, Coverpoint, Cross2,
    CrossBuildError, CrossSampleError,
};

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

/// Counter functional-coverage errors.
#[derive(Debug, Error)]
pub enum CounterCoverageError {
    /// Coverpoint construction failed.
    #[error(transparent)]
    Build(#[from] CoverageBuildError),
    /// Cross construction failed.
    #[error(transparent)]
    CrossBuild(#[from] CrossBuildError),
    /// Coverpoint sampling failed.
    #[error(transparent)]
    Sample(#[from] CoverageSampleError),
    /// Cross sampling failed.
    #[error(transparent)]
    CrossSample(#[from] CrossSampleError),
    /// Coverage-group validation failed.
    #[error(transparent)]
    Group(#[from] CoverageGroupError),
}

/// Functional coverage for one counter DUT instance.
pub struct CounterCoverage {
    /// Stable definition and instance identity.
    instance: CoverageGroupInstance,
    /// Reset, hold, and count operation.
    operation: Coverpoint<CounterOperation>,
    /// Sampled counter output region.
    count_region: Coverpoint<u8>,
    /// Sampled output parity, where `false` is even and `true` is odd.
    parity: Coverpoint<bool>,
    /// Operation against output parity.
    operation_x_parity: Cross2,
    /// Operation against output region.
    operation_x_count_region: Cross2,
}

impl CounterCoverage {
    /// Constructs functional coverage for the counter DUT.
    pub fn new() -> Result<Self, CounterCoverageError> {
        let operation = Coverpoint::builder("operation")
            .bin(Bin::value("hold", CounterOperation::Hold))
            .bin(Bin::value("count", CounterOperation::Count))
            .ignore_bin(Bin::value("reset", CounterOperation::Reset))
            .illegal_bin(Bin::value("invalid", CounterOperation::Invalid))
            .build()?;

        let count_region = Coverpoint::builder("count_region")
            .bin(Bin::value("zero", 0_u8))
            .bin(Bin::inclusive_range("low", 1_u8, 15_u8))
            .bin(Bin::inclusive_range("medium", 16_u8, 63_u8))
            .bin(Bin::inclusive_range("high", 64_u8, 254_u8))
            .bin(Bin::value("maximum", u8::MAX))
            .build()?;

        let parity = Coverpoint::builder("parity")
            .bin(Bin::value("even", false))
            .bin(Bin::value("odd", true))
            .build()?;

        let operation_x_parity =
            Cross2::builder("operation_x_parity", &operation, &parity).build()?;
        let operation_x_count_region =
            Cross2::builder("operation_x_count_region", &operation, &count_region).build()?;
        let instance =
            CoverageGroupInstance::new_with_revision("counter_coverage", "dut.counter", 1)?;
        let coverage = Self {
            instance,
            operation,
            count_region,
            parity,
            operation_x_parity,
            operation_x_count_region,
        };

        coverage.validate()?;

        Ok(coverage)
    }

    /// Samples one successfully executed counter transaction.
    pub fn sample(
        &mut self,
        stimulus: CounterStimulus,
        observation: CounterObservation,
    ) -> Result<(), CounterCoverageError> {
        let operation = operation_for(stimulus);
        let count = observation.count();
        let odd = count & 1 != 0;

        let operation_sample = self.operation.sample(&operation)?;
        let count_region_sample = self.count_region.sample(&count)?;
        let parity_sample = self.parity.sample(&odd)?;

        self.operation_x_parity
            .sample(&operation_sample, &parity_sample)?;
        self.operation_x_count_region
            .sample(&operation_sample, &count_region_sample)?;

        Ok(())
    }
}

impl CoverageGroup for CounterCoverage {
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        visitor.visit(CoverageItemRef::coverpoint(&self.operation));
        visitor.visit(CoverageItemRef::coverpoint(&self.count_region));
        visitor.visit(CoverageItemRef::coverpoint(&self.parity));
        visitor.visit(CoverageItemRef::cross2(&self.operation_x_parity));
        visitor.visit(CoverageItemRef::cross2(&self.operation_x_count_region));
    }
}

/// Derives the operation represented by valid counter stimulus.
const fn operation_for(stimulus: CounterStimulus) -> CounterOperation {
    if !stimulus.reset_n() {
        CounterOperation::Reset
    } else if stimulus.enable() {
        CounterOperation::Count
    } else {
        CounterOperation::Hold
    }
}

#[cfg(test)]
mod tests {
    use vvm::{CoverageGroup, CoverageGroupVisitor, CoverageItemRef};

    use super::{CounterCoverage, CounterOperation, operation_for};
    use crate::verification::{CounterObservation, CounterStimulus};

    #[test]
    fn counter_coverage_builds_and_validates() -> Result<(), Box<dyn std::error::Error>> {
        let coverage = CounterCoverage::new()?;

        coverage.validate()?;
        assert_eq!(coverage.instance().definition_name(), "counter_coverage");
        assert_eq!(coverage.instance().definition_revision(), 1);
        assert_eq!(coverage.instance().instance_path(), "dut.counter");

        Ok(())
    }

    #[test]
    fn operation_derivation_uses_valid_stimulus() {
        assert_eq!(
            operation_for(CounterStimulus::new(false, true)),
            CounterOperation::Reset
        );
        assert_eq!(
            operation_for(CounterStimulus::new(true, false)),
            CounterOperation::Hold
        );
        assert_eq!(
            operation_for(CounterStimulus::new(true, true)),
            CounterOperation::Count
        );
    }

    #[test]
    fn regions_parity_and_crosses_are_sampled() -> Result<(), Box<dyn std::error::Error>> {
        let mut coverage = CounterCoverage::new()?;
        coverage.sample(
            CounterStimulus::new(false, false),
            CounterObservation::new(0),
        )?;
        coverage.sample(
            CounterStimulus::new(true, false),
            CounterObservation::new(1),
        )?;
        coverage.sample(
            CounterStimulus::new(true, true),
            CounterObservation::new(64),
        )?;
        coverage.sample(
            CounterStimulus::new(true, true),
            CounterObservation::new(255),
        )?;

        assert_eq!(coverage.operation_x_parity.sample_count(), 4);
        assert_eq!(coverage.operation_x_parity.skipped_sample_count(), 1);
        assert_eq!(coverage.operation_x_count_region.sample_count(), 4);
        assert_eq!(coverage.operation_x_count_region.skipped_sample_count(), 1);
        assert!(
            coverage
                .count_region
                .bins()
                .iter()
                .any(|bin| bin.name() == "zero" && bin.hits() == 1)
        );
        assert!(
            coverage
                .parity
                .bins()
                .iter()
                .any(|bin| bin.name() == "odd" && bin.hits() == 2)
        );

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

        let coverage = CounterCoverage::new()?;
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
