use vvm::coverage::{
    Bin, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor, CoverageItemRef, Coverpoint,
};
use vvm::test::TestContext;
use vvm::testbench::TestResult;
use vvm::timing::SimulationTime;

const COVERAGE_REPLAY: vvm::random::ReplayToken =
    vvm::random::ReplayToken::new(vvm::random::Seed::new(0x44));

struct ManualCoverage {
    instance: CoverageGroupInstance,
    value: Coverpoint<u8>,
}

impl CoverageGroup for ManualCoverage {
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        visitor.visit(CoverageItemRef::coverpoint(&self.value));
    }
}

#[vvm::test(replay(default = COVERAGE_REPLAY), coverage)]
/// Captures one minimal packaged coverage artifact.
fn packaged_manual_result_with_coverage(
    context: &mut TestContext,
) -> Result<TestResult<(), String, String>, String> {
    let mut value = Coverpoint::builder("value")
        .bin(Bin::value("one", 1_u8))
        .build()
        .map_err(|error| error.to_string())?;

    value.sample(&1).map_err(|error| error.to_string())?;

    let coverage = ManualCoverage {
        instance: CoverageGroupInstance::new("manual", "dut.manual")
            .map_err(|error| error.to_string())?,
        value,
    };

    context
        .capture_coverage(&coverage)
        .map_err(|error| error.to_string())?;

    Ok(TestResult::completed(
        SimulationTime::ZERO,
        SimulationTime::from_ticks(1),
        context.config().replay_token(),
    ))
}
