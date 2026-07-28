use vvm::test::TestRunConfig;
use vvm::testbench::{ExactScoreboard, Testbench};

use crate::signed_adder::SignedAdder;
use crate::verification::{
    Result, SignedAdderClock, SignedAdderObservation, SignedAdderReferenceModel,
    SignedAdderTestResult, exhaustive_signed_adder_sequence, signed_adder_smoke_sequence,
};

/// Boundary-focused signed-adder verification with VCD tracing.
#[vvm::test(trace)]
fn signed_adder_smoke(config: &TestRunConfig) -> Result<SignedAdderTestResult> {
    let mut dut = SignedAdder::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(signed_adder_smoke_sequence())
        .with_reference_model(SignedAdderReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(SignedAdderClock)
        .run::<SignedAdderObservation>();

    Ok(result)
}

/// Exhaustively verifies all 65,536 signed eight-bit operand pairs.
#[vvm::test]
fn signed_adder_exhaustive() -> Result<SignedAdderTestResult> {
    let dut = SignedAdder::new()?;

    let result = Testbench::new(dut)
        .with_sequence(exhaustive_signed_adder_sequence())
        .with_reference_model(SignedAdderReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(SignedAdderClock)
        .run::<SignedAdderObservation>();

    Ok(result)
}
