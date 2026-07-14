use vvm::{Bits, Clock, Drive, Mismatch, ReferenceModel, Sample, SignedBits, TestResult};

use crate::wide_transform::WideTransformError;

/// Width used by the wide-transform DUT.
const WIDE_WIDTH: usize = 129;

/// Unsigned 129-bit value type used by the example.
pub type WideValue = Bits<WIDE_WIDTH>;

/// Signed 129-bit value type used by the example.
pub type SignedWideValue = SignedBits<WIDE_WIDTH>;

/// Inputs applied during one wide-transform cycle.
#[derive(Debug, Clone, PartialEq, Eq, Drive)]
#[vvm(dut = crate::wide_transform::WideTransform)]
pub struct WideTransformStimulus {
    /// Unsigned wide input.
    #[vvm(port)]
    value: WideValue,

    /// Signed wide input.
    #[vvm(port)]
    signed_value: SignedWideValue,
}

impl WideTransformStimulus {
    /// Creates one wide-transform stimulus.
    #[must_use]
    pub fn new(value: WideValue, signed_value: SignedWideValue) -> Self {
        Self {
            value,
            signed_value,
        }
    }

    /// Returns the unsigned input.
    #[must_use]
    pub const fn value(&self) -> &WideValue {
        &self.value
    }

    /// Returns the signed input.
    #[must_use]
    pub const fn signed_value(&self) -> &SignedWideValue {
        &self.signed_value
    }
}

/// Outputs sampled after the active clock edge.
#[derive(Debug, Clone, PartialEq, Eq, Sample)]
#[vvm(dut = crate::wide_transform::WideTransform)]
pub struct WideTransformObservation {
    /// Shifted unsigned wide output.
    #[vvm(port)]
    shifted: WideValue,

    /// Low 32-bit tap of the unsigned input.
    #[vvm(port)]
    low_word: u32,

    /// High-bit tap of the unsigned input.
    #[vvm(port)]
    high_bit: bool,

    /// Mirrored signed wide value.
    #[vvm(port)]
    signed_mirror: SignedWideValue,

    /// Sign bit of the signed wide value.
    #[vvm(port)]
    signed_sign: bool,
}

impl WideTransformObservation {
    /// Creates an expected observation.
    #[must_use]
    pub const fn new(
        shifted: WideValue,
        low_word: u32,
        high_bit: bool,
        signed_mirror: SignedWideValue,
        signed_sign: bool,
    ) -> Self {
        Self {
            shifted,
            low_word,
            high_bit,
            signed_mirror,
            signed_sign,
        }
    }

    /// Returns the shifted value.
    #[must_use]
    pub const fn shifted(&self) -> &WideValue {
        &self.shifted
    }

    /// Returns the low-word tap.
    #[must_use]
    pub const fn low_word(&self) -> u32 {
        self.low_word
    }

    /// Returns the high-bit tap.
    #[must_use]
    pub const fn high_bit(&self) -> bool {
        self.high_bit
    }

    /// Returns the signed mirror.
    #[must_use]
    pub const fn signed_mirror(&self) -> &SignedWideValue {
        &self.signed_mirror
    }

    /// Returns the signed sign-bit tap.
    #[must_use]
    pub const fn signed_sign(&self) -> bool {
        self.signed_sign
    }
}

/// Clock driver for the generated wide-transform DUT.
#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::wide_transform::WideTransform,
    clock = "clk"
)]
pub struct WideTransformClock;

/// Wide-transform reference model.
#[derive(Debug, Clone, Copy, Default)]
pub struct WideTransformReferenceModel;

impl ReferenceModel<WideTransformStimulus> for WideTransformReferenceModel {
    type Expected = WideTransformObservation;

    fn predict(&mut self, stimulus: &WideTransformStimulus) -> Self::Expected {
        let shifted = shift_left_one(stimulus.value());

        let low_word = stimulus.value().words_le().first().copied().unwrap_or(0);

        let high_bit = stimulus.value().bit(128).unwrap_or(false);

        let signed_mirror = stimulus.signed_value().clone();

        let signed_sign = stimulus.signed_value().is_negative();

        WideTransformObservation::new(shifted, low_word, high_bit, signed_mirror, signed_sign)
    }
}

/// Wide-transform scoreboard mismatch.
pub type WideTransformMismatch = Mismatch<WideTransformObservation, WideTransformObservation>;

/// Complete result of one wide-transform verification run.
pub type WideTransformTestResult =
    TestResult<WideTransformStimulus, WideTransformMismatch, WideTransformError>;

/// Wide-transform setup result.
pub type Result<T> = std::result::Result<T, WideTransformError>;

/// Boundary-focused wide-transform sequence.
pub fn wide_transform_smoke_sequence()
-> Result<impl ExactSizeIterator<Item = WideTransformStimulus>> {
    Ok([
        WideTransformStimulus::new(wide_value([0, 0, 0, 0, 0])?, signed_value([0, 0, 0, 0, 0])?),
        WideTransformStimulus::new(wide_value([1, 0, 0, 0, 0])?, signed_value([1, 0, 0, 0, 0])?),
        WideTransformStimulus::new(
            wide_value([0x8000_0000, 0, 0, 0, 0])?,
            signed_value([
                0xffff_ffff,
                0xffff_ffff,
                0xffff_ffff,
                0xffff_ffff,
                0x0000_0001,
            ])?,
        ),
        WideTransformStimulus::new(
            wide_value([
                0x0123_4567,
                0x89ab_cdef,
                0x1357_9bdf,
                0x2468_ace0,
                0x0000_0001,
            ])?,
            signed_value([
                0x7654_3210,
                0xfedc_ba98,
                0x0246_8ace,
                0x1357_9bdf,
                0x0000_0001,
            ])?,
        ),
        WideTransformStimulus::new(
            wide_value([u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX])?,
            signed_value([u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX])?,
        ),
    ]
    .into_iter())
}

/// Creates a canonical unsigned wide value from words.
fn wide_value(words: [u32; 5]) -> Result<WideValue> {
    WideValue::from_words_le(words).map_err(|_error| WideTransformError::WidePortTransferFailed)
}

/// Creates a canonical signed wide value from words.
fn signed_value(words: [u32; 5]) -> Result<SignedWideValue> {
    SignedWideValue::from_words_le(words)
        .map_err(|_error| WideTransformError::WidePortTransferFailed)
}

/// Computes the DUT's unsigned one-bit left shift.
fn shift_left_one(value: &WideValue) -> WideValue {
    let mut carry = 0_u32;

    let shifted_words = value
        .words_le()
        .iter()
        .map(|word| {
            let next_carry = word >> 31;
            let shifted = word.wrapping_shl(1) | carry;

            carry = next_carry;

            shifted
        })
        .collect::<Vec<_>>();

    // Structurally valid: source and destination are the same declared width.
    WideValue::from_words_le(shifted_words).unwrap_or_else(|_error| WideValue::zero())
}

#[cfg(test)]
mod tests {
    use vvm::ReferenceModel;

    use super::{WideTransformReferenceModel, WideTransformStimulus, signed_value, wide_value};

    #[test]
    fn reference_model_shifts_across_word_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let stimulus = WideTransformStimulus::new(
            wide_value([0x8000_0000, 0, 0, 0, 0])?,
            signed_value([0, 0, 0, 0, 0])?,
        );

        let mut model = WideTransformReferenceModel;

        let observation = model.predict(&stimulus);

        assert_eq!(observation.shifted().words_le(), &[0, 1, 0, 0, 0,]);

        assert_eq!(observation.low_word(), 0x8000_0000,);

        assert!(!observation.high_bit());

        Ok(())
    }

    #[test]
    fn reference_model_preserves_high_bit_and_masks_storage()
    -> Result<(), Box<dyn std::error::Error>> {
        let stimulus = WideTransformStimulus::new(
            wide_value([u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX])?,
            signed_value([u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX])?,
        );

        let mut model = WideTransformReferenceModel;

        let observation = model.predict(&stimulus);

        assert_eq!(
            observation.shifted().words_le(),
            &[0xffff_fffe, u32::MAX, u32::MAX, u32::MAX, 0x0000_0001],
        );

        assert_eq!(observation.low_word(), u32::MAX,);

        assert!(observation.high_bit());
        assert!(observation.signed_sign());

        assert_eq!(
            observation.signed_mirror().words_le(),
            &[u32::MAX, u32::MAX, u32::MAX, u32::MAX, 0x0000_0001],
        );

        Ok(())
    }
}
