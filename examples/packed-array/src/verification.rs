use vvm::testbench::{Mismatch, ReferenceModel, TestResult};
use vvm::{Clock, Drive, Sample};

use crate::packed_array_ports::{PackedArrayPortsError, PackedBytes, PackedBytesOut};

/// Inputs applied during one packed-array cycle.
#[derive(Debug, Clone, PartialEq, Eq, Drive)]
#[vvm(dut = crate::packed_array_ports::PackedArrayPorts)]
pub struct PackedArrayStimulus {
    /// Packed-array input value.
    #[vvm(port)]
    packed_bytes: PackedBytes,
}

impl PackedArrayStimulus {
    /// Creates one packed-array stimulus.
    #[must_use]
    pub fn new(packed_bytes: PackedBytes) -> Self {
        Self { packed_bytes }
    }

    /// Returns the packed-array input.
    #[must_use]
    pub const fn packed_bytes(&self) -> &PackedBytes {
        &self.packed_bytes
    }
}

/// Outputs sampled after the active clock edge.
#[derive(Debug, Clone, PartialEq, Eq, Sample)]
#[vvm(dut = crate::packed_array_ports::PackedArrayPorts)]
pub struct PackedArrayObservation {
    /// Mirrored packed-array output.
    #[vvm(port)]
    packed_bytes_out: PackedBytesOut,

    /// HDL element `packed_bytes[0]` tap.
    #[vvm(port)]
    first_byte: u8,

    /// HDL element `packed_bytes[3]` tap.
    #[vvm(port)]
    last_byte: u8,
}

impl PackedArrayObservation {
    /// Creates an expected observation.
    #[must_use]
    pub fn new(packed_bytes_out: PackedBytesOut, first_byte: u8, last_byte: u8) -> Self {
        Self {
            packed_bytes_out,
            first_byte,
            last_byte,
        }
    }
}

/// Clock driver for the generated packed-array DUT.
#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::packed_array_ports::PackedArrayPorts,
    clock = "clk"
)]
pub struct PackedArrayClock;

/// Packed-array reference model.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackedArrayReferenceModel;

impl ReferenceModel<PackedArrayStimulus> for PackedArrayReferenceModel {
    type Expected = PackedArrayObservation;

    fn predict(&mut self, stimulus: &PackedArrayStimulus) -> Self::Expected {
        let first_byte = stimulus.packed_bytes().element(0).unwrap_or(0);
        let last_byte = stimulus.packed_bytes().element(3).unwrap_or(0);
        let packed_bytes_out = PackedBytesOut::from_bits(stimulus.packed_bytes().bits().clone());

        PackedArrayObservation::new(packed_bytes_out, first_byte, last_byte)
    }
}

/// Packed-array scoreboard mismatch.
pub type PackedArrayMismatch = Mismatch<PackedArrayObservation, PackedArrayObservation>;

/// Complete result of one packed-array verification run.
pub type PackedArrayTestResult =
    TestResult<PackedArrayStimulus, PackedArrayMismatch, PackedArrayPortsError>;

/// Packed-array setup result.
pub type Result<T> = std::result::Result<T, PackedArrayPortsError>;

/// Boundary-focused packed-array smoke sequence.
pub fn packed_array_smoke_sequence() -> Result<impl ExactSizeIterator<Item = PackedArrayStimulus>> {
    Ok([
        PackedArrayStimulus::new(packed_bytes([(0, 0x12), (3, 0xa5)])?),
        PackedArrayStimulus::new(packed_bytes([(0, 0xff), (1, 0x34), (3, 0x80)])?),
    ]
    .into_iter())
}

/// Builds one valid packed-array value from HDL-indexed byte assignments.
fn packed_bytes<const N: usize>(assignments: [(i64, u8); N]) -> Result<PackedBytes> {
    let mut value = PackedBytes::zero();

    for (index, element) in assignments {
        value
            .set_element(index, element)
            .map_err(|_error| PackedArrayPortsError::PackedLayoutFailed)?;
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use vvm::testbench::ReferenceModel;

    use super::{PackedArrayReferenceModel, PackedArrayStimulus, packed_bytes};

    #[test]
    fn reference_model_uses_hdl_index_order() -> Result<(), Box<dyn std::error::Error>> {
        let stimulus = PackedArrayStimulus::new(packed_bytes([(0, 0x11), (3, 0xaa)])?);
        let mut model = PackedArrayReferenceModel;

        let observation = model.predict(&stimulus);

        assert_eq!(observation.first_byte, 0x11);
        assert_eq!(observation.last_byte, 0xaa);
        assert_eq!(observation.packed_bytes_out.element(0)?, 0x11);
        assert_eq!(observation.packed_bytes_out.element(3)?, 0xaa);

        Ok(())
    }
}
