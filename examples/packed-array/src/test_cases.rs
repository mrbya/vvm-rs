use vvm::test::TestRunConfig;
use vvm::testbench::{ExactScoreboard, Testbench};

use crate::packed_array_ports::PackedArrayPorts;
use crate::verification::{
    PackedArrayClock, PackedArrayObservation, PackedArrayReferenceModel, PackedArrayTestResult,
    Result, packed_array_smoke_sequence,
};

/// HDL-indexed packed-array verification with VCD tracing.
#[vvm::test(trace)]
fn packed_array_smoke(config: &TestRunConfig) -> Result<PackedArrayTestResult> {
    let mut dut = PackedArrayPorts::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(packed_array_smoke_sequence()?)
        .with_reference_model(PackedArrayReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(PackedArrayClock)
        .run::<PackedArrayObservation>();

    Ok(result)
}
