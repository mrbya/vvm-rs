use vvm::testbench::{Mismatch, ReferenceModel, TestResult};
use vvm::{Clock, Drive, Sample};

use crate::packed_struct_ports::{
    PackedStructPortsError, Packet, PacketOut, WidePacket, WidePacketOut,
};

/// Inputs applied during one packed-struct cycle.
#[derive(Debug, Clone, PartialEq, Eq, Drive)]
#[vvm(dut = crate::packed_struct_ports::PackedStructPorts)]
pub struct PackedStructStimulus {
    /// Packed-struct input value.
    #[vvm(port)]
    packet: Packet,

    /// Wide packed-struct input value.
    #[vvm(port)]
    wide_packet: WidePacket,

    expected_opcode: u8,
    expected_valid: bool,
    expected_delta: i8,
    expected_flags: u8,
    expected_payload: u16,
    expected_wide_tag: u8,
    expected_wide_payload: ::vvm::packed::SignedBits<129>,
}

impl PackedStructStimulus {
    /// Creates one packed-struct stimulus.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        packet: Packet,
        wide_packet: WidePacket,
        expected_opcode: u8,
        expected_valid: bool,
        expected_delta: i8,
        expected_flags: u8,
        expected_payload: u16,
        expected_wide_tag: u8,
        expected_wide_payload: ::vvm::packed::SignedBits<129>,
    ) -> Self {
        Self {
            packet,
            wide_packet,
            expected_opcode,
            expected_valid,
            expected_delta,
            expected_flags,
            expected_payload,
            expected_wide_tag,
            expected_wide_payload,
        }
    }
}

/// Outputs sampled after the active clock edge.
#[derive(Debug, Clone, PartialEq, Eq, Sample)]
#[vvm(dut = crate::packed_struct_ports::PackedStructPorts)]
#[allow(clippy::struct_field_names)]
pub struct PackedStructObservation {
    /// Mirrored packed-struct output.
    #[vvm(port)]
    packet_out: PacketOut,

    /// Mirrored wide packed-struct output.
    #[vvm(port)]
    wide_packet_out: WidePacketOut,

    /// Scalar output taps.
    #[vvm(port)]
    opcode_out: u8,
    #[vvm(port)]
    valid_out: bool,
    #[vvm(port)]
    delta_out: i8,
    #[vvm(port)]
    flags_out: u8,
    #[vvm(port)]
    payload_out: u16,

    /// Wide field taps.
    #[vvm(port)]
    wide_tag_out: u8,
    #[vvm(port)]
    wide_payload_out: ::vvm::packed::SignedBits<129>,
}

impl PackedStructObservation {
    /// Creates an expected observation.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        packet_out: PacketOut,
        wide_packet_out: WidePacketOut,
        opcode_out: u8,
        valid_out: bool,
        delta_out: i8,
        flags_out: u8,
        payload_out: u16,
        wide_tag_out: u8,
        wide_payload_out: ::vvm::packed::SignedBits<129>,
    ) -> Self {
        Self {
            packet_out,
            wide_packet_out,
            opcode_out,
            valid_out,
            delta_out,
            flags_out,
            payload_out,
            wide_tag_out,
            wide_payload_out,
        }
    }
}

/// Clock driver for the generated packed-struct DUT.
#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::packed_struct_ports::PackedStructPorts,
    clock = "clk"
)]
pub struct PackedStructClock;

/// Packed-struct reference model.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackedStructReferenceModel;

impl ReferenceModel<PackedStructStimulus> for PackedStructReferenceModel {
    type Expected = PackedStructObservation;

    fn predict(&mut self, stimulus: &PackedStructStimulus) -> Self::Expected {
        PackedStructObservation::new(
            PacketOut::from_bits(stimulus.packet.bits().clone()),
            WidePacketOut::from_bits(stimulus.wide_packet.bits().clone()),
            stimulus.expected_opcode,
            stimulus.expected_valid,
            stimulus.expected_delta,
            stimulus.expected_flags,
            stimulus.expected_payload,
            stimulus.expected_wide_tag,
            stimulus.expected_wide_payload.clone(),
        )
    }
}

/// Packed-struct scoreboard mismatch.
pub type PackedStructMismatch = Mismatch<PackedStructObservation, PackedStructObservation>;

/// Complete result of one packed-struct verification run.
pub type PackedStructTestResult =
    TestResult<PackedStructStimulus, PackedStructMismatch, PackedStructPortsError>;

/// Packed-struct setup result.
pub type Result<T> = std::result::Result<T, PackedStructPortsError>;

/// Packed-struct smoke sequence.
pub fn packed_struct_smoke_sequence() -> Result<impl ExactSizeIterator<Item = PackedStructStimulus>>
{
    let signed_payload_a = signed_payload([0x89ab_cdef, 0x0123_4567, 0, 0, 1])?;
    let signed_payload_b = signed_payload([u32::MAX, 0x8000_0000, u32::MAX, 0, 1])?;

    Ok([
        PackedStructStimulus::new(
            packet(0x0a, true, -17, 0x05, 0xcafe)?,
            wide_packet(0x55, &signed_payload_a)?,
            0x0a,
            true,
            -17,
            0x05,
            0xcafe,
            0x55,
            signed_payload_a,
        ),
        PackedStructStimulus::new(
            packet(0x03, false, 12, 0x0f, 0x1234)?,
            wide_packet(0x12, &signed_payload_b)?,
            0x03,
            false,
            12,
            0x0f,
            0x1234,
            0x12,
            signed_payload_b,
        ),
    ]
    .into_iter())
}

/// Builds one packed packet value.
fn packet(opcode: u8, valid: bool, delta: i8, flags: u8, payload: u16) -> Result<Packet> {
    let mut packet = Packet::zero();
    packet
        .set_opcode(opcode)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    packet
        .set_valid(valid)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    packet
        .set_delta(delta)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    packet
        .set_flags(flags)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    packet
        .set_payload(payload)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    Ok(packet)
}

/// Builds one wide packed packet value.
fn wide_packet(tag: u8, payload: &::vvm::packed::SignedBits<129>) -> Result<WidePacket> {
    let mut packet = WidePacket::zero();
    packet
        .set_tag(tag)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    packet
        .set_payload(payload)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)?;
    Ok(packet)
}

/// Creates a canonical signed payload from words.
fn signed_payload(words: [u32; 5]) -> Result<::vvm::packed::SignedBits<129>> {
    ::vvm::packed::SignedBits::<129>::from_words_le(words)
        .map_err(|_error| PackedStructPortsError::PackedLayoutFailed)
}

#[cfg(test)]
mod tests {
    use vvm::testbench::ReferenceModel;

    use super::{
        PackedStructReferenceModel, PackedStructStimulus, packet, signed_payload, wide_packet,
    };

    #[test]
    fn reference_model_preserves_precomputed_field_values() -> Result<(), Box<dyn std::error::Error>>
    {
        let payload = signed_payload([1, 2, 3, 4, 1])?;
        let stimulus = PackedStructStimulus::new(
            packet(0x0a, true, -17, 0x05, 0xcafe)?,
            wide_packet(0x55, &payload)?,
            0x0a,
            true,
            -17,
            0x05,
            0xcafe,
            0x55,
            payload.clone(),
        );
        let mut model = PackedStructReferenceModel;

        let observation = model.predict(&stimulus);

        assert_eq!(observation.opcode_out, 0x0a);
        assert!(observation.valid_out);
        assert_eq!(observation.delta_out, -17);
        assert_eq!(observation.payload_out, 0xcafe);
        assert_eq!(observation.wide_tag_out, 0x55);
        assert_eq!(observation.wide_payload_out, payload);

        Ok(())
    }

    #[test]
    fn packet_field_updates_preserve_unrelated_bits() -> Result<(), Box<dyn std::error::Error>> {
        let mut packet = packet(0x0a, true, -17, 0x05, 0x1234)?;
        let opcode_before = packet.opcode()?;
        let valid_before = packet.valid()?;

        packet.set_payload(0xcafe)?;

        assert_eq!(packet.opcode()?, opcode_before);
        assert_eq!(packet.valid()?, valid_before);
        assert_eq!(packet.payload()?, 0xcafe);

        Ok(())
    }

    #[test]
    fn wide_field_updates_preserve_adjacent_scalar_bits() -> Result<(), Box<dyn std::error::Error>>
    {
        let first = signed_payload([0x1111_2222, 0x3333_4444, 0x5555_6666, 0x7777_8888, 1])?;
        let second = signed_payload([0xaaaa_bbbb, 0xcccc_dddd, 0xeeee_ffff, 0x1234_5678, 0])?;
        let mut packet = wide_packet(0x55, &first)?;

        packet.set_payload(&second)?;

        assert_eq!(packet.tag()?, 0x55);
        assert_eq!(packet.payload()?, second);

        Ok(())
    }
}
