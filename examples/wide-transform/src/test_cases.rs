use vvm::{ExactScoreboard, TestRunConfig, Testbench};

use crate::verification::{
    Result, WideTransformClock, WideTransformObservation, WideTransformReferenceModel,
    WideTransformTestResult, wide_transform_smoke_sequence,
};
use crate::wide_transform::WideTransform;

/// Boundary-focused wide-port verification with VCD tracing.
#[vvm::test(trace)]
fn wide_transform_smoke(config: &TestRunConfig) -> Result<WideTransformTestResult> {
    let mut dut = WideTransform::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(wide_transform_smoke_sequence()?)
        .with_reference_model(WideTransformReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(WideTransformClock)
        .run::<WideTransformObservation>();

    Ok(result)
}
