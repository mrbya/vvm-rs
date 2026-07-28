use vvm::testbench::{Mismatch, ReferenceModel, TestResult};
use vvm::{Clock, Drive, Sample};

use crate::packed_enum_ports::{
    PackedEnumPortsError, SignedState, SignedStateOut, SignedStateVariant, State, StateOut,
    StateVariant,
};

/// Inputs applied during one packed-enum cycle.
#[derive(Debug, Clone, PartialEq, Eq, Drive)]
#[vvm(dut = crate::packed_enum_ports::PackedEnumPorts)]
pub struct PackedEnumStimulus {
    #[vvm(port)]
    state: State,

    #[vvm(port)]
    signed_state: SignedState,

    expected_state_raw: u8,
    expected_signed_raw: i8,
}

impl PackedEnumStimulus {
    /// Creates one packed-enum stimulus.
    #[must_use]
    pub fn new(
        state: State,
        signed_state: SignedState,
        expected_state_raw: u8,
        expected_signed_raw: i8,
    ) -> Self {
        Self {
            state,
            signed_state,
            expected_state_raw,
            expected_signed_raw,
        }
    }
}

/// Outputs sampled after the active clock edge.
#[derive(Debug, Clone, PartialEq, Eq, Sample)]
#[vvm(dut = crate::packed_enum_ports::PackedEnumPorts)]
#[allow(clippy::struct_field_names)]
pub struct PackedEnumObservation {
    #[vvm(port)]
    state_out: StateOut,
    #[vvm(port)]
    state_raw_out: u8,
    #[vvm(port)]
    signed_state_out: SignedStateOut,
    #[vvm(port)]
    signed_state_raw_out: i8,
}

impl PackedEnumObservation {
    /// Creates an expected observation.
    #[must_use]
    pub fn new(
        state_out: StateOut,
        state_raw_out: u8,
        signed_state_out: SignedStateOut,
        signed_state_raw_out: i8,
    ) -> Self {
        Self {
            state_out,
            state_raw_out,
            signed_state_out,
            signed_state_raw_out,
        }
    }
}

/// Clock driver for the generated packed-enum DUT.
#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::packed_enum_ports::PackedEnumPorts,
    clock = "clk"
)]
pub struct PackedEnumClock;

/// Packed-enum reference model.
#[derive(Debug, Clone, Copy, Default)]
pub struct PackedEnumReferenceModel;

impl ReferenceModel<PackedEnumStimulus> for PackedEnumReferenceModel {
    type Expected = PackedEnumObservation;

    fn predict(&mut self, stimulus: &PackedEnumStimulus) -> Self::Expected {
        PackedEnumObservation::new(
            StateOut::from_bits(stimulus.state.bits().clone()),
            stimulus.expected_state_raw,
            SignedStateOut::from_bits(stimulus.signed_state.bits().clone()),
            stimulus.expected_signed_raw,
        )
    }
}

/// Packed-enum scoreboard mismatch.
pub type PackedEnumMismatch = Mismatch<PackedEnumObservation, PackedEnumObservation>;

/// Complete result of one packed-enum verification run.
pub type PackedEnumTestResult =
    TestResult<PackedEnumStimulus, PackedEnumMismatch, PackedEnumPortsError>;

/// Packed-enum setup result.
pub type Result<T> = std::result::Result<T, PackedEnumPortsError>;

/// Packed-enum smoke sequence.
pub fn packed_enum_smoke_sequence() -> Result<impl ExactSizeIterator<Item = PackedEnumStimulus>> {
    let known = State::from_variant(StateVariant::STATE_BUSY);
    assert_eq!(
        known
            .raw()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        2
    );
    assert_eq!(
        known
            .variant()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        Some(StateVariant::STATE_BUSY)
    );
    assert_eq!(
        known
            .name()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        Some("STATE_BUSY")
    );
    assert!(
        known
            .is_known()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?
    );

    let unknown = State::from_raw(3);
    assert_eq!(
        unknown
            .raw()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        3
    );
    assert_eq!(
        unknown
            .variant()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        None
    );
    assert_eq!(
        unknown
            .name()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        None
    );
    assert!(
        !unknown
            .is_known()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?
    );

    let signed_neg = SignedState::from_variant(SignedStateVariant::SIGNED_NEG);
    assert_eq!(
        signed_neg
            .raw()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        0x0D
    );
    assert_eq!(
        signed_neg
            .variant()
            .map_err(|_error| PackedEnumPortsError::PackedLayoutFailed)?,
        Some(SignedStateVariant::SIGNED_NEG)
    );

    Ok([
        PackedEnumStimulus::new(
            known,
            SignedState::from_variant(SignedStateVariant::SIGNED_ZERO),
            2,
            0,
        ),
        PackedEnumStimulus::new(
            unknown,
            SignedState::from_variant(SignedStateVariant::SIGNED_NEG),
            3,
            -3,
        ),
        PackedEnumStimulus::new(
            State::from_variant(StateVariant::STATE_DONE),
            signed_neg,
            5,
            -3,
        ),
        PackedEnumStimulus::new(State::from_raw(0), SignedState::from_raw(0), 0, 0),
        PackedEnumStimulus::new(
            State::from_raw(u64::MAX),
            SignedState::from_raw(u64::MAX),
            7,
            -1,
        ),
    ]
    .into_iter())
}

#[cfg(test)]
mod tests {
    use vvm::testbench::ReferenceModel;

    use super::{PackedEnumReferenceModel, packed_enum_smoke_sequence};
    use crate::packed_enum_ports::{SignedState, SignedStateVariant, State, StateVariant};

    #[test]
    fn known_unsigned_variant_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        let state = State::from_variant(StateVariant::STATE_BUSY);

        assert_eq!(state.raw()?, 2);
        assert_eq!(state.variant()?, Some(StateVariant::STATE_BUSY));
        assert_eq!(state.name()?, Some("STATE_BUSY"));
        assert!(state.is_known()?);

        Ok(())
    }

    #[test]
    fn unknown_unsigned_variant_is_preserved() -> Result<(), Box<dyn std::error::Error>> {
        let state = State::from_raw(3);

        assert_eq!(state.raw()?, 3);
        assert_eq!(state.variant()?, None);
        assert_eq!(state.name()?, None);
        assert!(!state.is_known()?);

        Ok(())
    }

    #[test]
    fn signed_negative_variant_preserves_raw_bits() -> Result<(), Box<dyn std::error::Error>> {
        let state = SignedState::from_variant(SignedStateVariant::SIGNED_NEG);

        assert_eq!(state.raw()?, 0x0D);
        assert_eq!(state.variant()?, Some(SignedStateVariant::SIGNED_NEG));

        Ok(())
    }

    #[test]
    fn unsigned_raw_construction_truncates_to_width() -> Result<(), Box<dyn std::error::Error>> {
        let state = State::from_raw(0xFF);

        assert_eq!(state.raw()?, 0x07);

        Ok(())
    }

    #[test]
    fn reference_model_mirrors_bits_and_raw_taps() -> Result<(), Box<dyn std::error::Error>> {
        let stimulus = packed_enum_smoke_sequence()?
            .next()
            .ok_or_else(|| std::io::Error::other("missing packed-enum stimulus"))?;
        let mut model = PackedEnumReferenceModel;

        let observation = model.predict(&stimulus);

        assert_eq!(observation.state_raw_out, 2);
        assert_eq!(observation.signed_state_raw_out, 0);

        Ok(())
    }
}
