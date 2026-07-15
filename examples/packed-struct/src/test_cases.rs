use vvm::{ExactScoreboard, TestRunConfig, Testbench};

use crate::packed_struct_ports::PackedStructPorts;
use crate::verification::{
    PackedStructClock, PackedStructObservation, PackedStructReferenceModel, PackedStructTestResult,
    Result, packed_struct_smoke_sequence,
};

/// Packed-struct verification with VCD tracing.
#[vvm::test(trace)]
fn packed_struct_smoke(config: &TestRunConfig) -> Result<PackedStructTestResult> {
    let mut dut = PackedStructPorts::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(packed_struct_smoke_sequence()?)
        .with_reference_model(PackedStructReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(PackedStructClock)
        .run::<PackedStructObservation>();

    Ok(result)
}
