use vvm::{ExactScoreboard, TestRunConfig, Testbench};

use crate::packed_enum_ports::PackedEnumPorts;
use crate::verification::{
    PackedEnumClock, PackedEnumObservation, PackedEnumReferenceModel, PackedEnumTestResult, Result,
    packed_enum_smoke_sequence,
};

/// Packed-enum verification with VCD tracing.
#[vvm::test(trace)]
fn packed_enum_smoke(config: &TestRunConfig) -> Result<PackedEnumTestResult> {
    let mut dut = PackedEnumPorts::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(packed_enum_smoke_sequence()?)
        .with_reference_model(PackedEnumReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(PackedEnumClock)
        .run::<PackedEnumObservation>();

    Ok(result)
}
