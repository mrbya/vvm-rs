use vvm::{Clock, Drive, Mismatch, ReferenceModel, Sample, TestResult};

use crate::signed_adder::SignedAdderError;

/// Inputs applied during one signed-adder cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Drive)]
#[vvm(dut = crate::signed_adder::SignedAdder)]
pub struct SignedAdderStimulus {
    /// Left signed operand.
    #[vvm(port)]
    lhs: i8,

    /// Right signed operand.
    #[vvm(port)]
    rhs: i8,
}

impl SignedAdderStimulus {
    /// Creates one signed-adder stimulus,
    #[must_use]
    pub const fn new(lhs: i8, rhs: i8) -> Self {
        Self { lhs, rhs }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sample)]
#[vvm(dut = crate::signed_adder::SignedAdder)]
pub struct SignedAdderObservation {
    /// Nine-bit signed addition result.
    #[vvm(port)]
    sum: i16,
}

impl SignedAdderObservation {
    /// Creates an expected signed-adder observation.
    #[must_use]
    pub const fn new(sum: i16) -> Self {
        Self { sum }
    }

    /// Returns the sampled sum.
    #[must_use]
    pub const fn sum(self) -> i16 {
        self.sum
    }
}

#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::signed_adder::SignedAdder,
    clock = "clk"
)]
pub struct SignedAdderClock;

/// Mathematical signed-adder reference model.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignedAdderReferenceModel;

impl ReferenceModel<SignedAdderStimulus> for SignedAdderReferenceModel {
    type Expected = SignedAdderObservation;

    fn predict(&mut self, stimulus: &SignedAdderStimulus) -> Self::Expected {
        let sum = i16::from(stimulus.lhs).wrapping_add(i16::from(stimulus.rhs));

        SignedAdderObservation::new(sum)
    }
}

/// Signed-adder scoreboard mismatch.
pub type SignedAdderMismatch = Mismatch<SignedAdderObservation, SignedAdderObservation>;

/// Complete result of one signed-adder verification run.
pub type SignedAdderTestResult =
    TestResult<SignedAdderStimulus, SignedAdderMismatch, SignedAdderError>;

/// Signed-adder setup result.
pub type Result<T> = std::result::Result<T, SignedAdderError>;

/// Deterministic signed-value sequence.
pub fn signed_adder_smoke_sequence() -> impl ExactSizeIterator<Item = SignedAdderStimulus> {
    [
        SignedAdderStimulus::new(0, 0),
        SignedAdderStimulus::new(1, -1),
        SignedAdderStimulus::new(-1, -1),
        SignedAdderStimulus::new(i8::MAX, i8::MAX),
        SignedAdderStimulus::new(i8::MIN, i8::MIN),
        SignedAdderStimulus::new(i8::MIN, i8::MAX),
        SignedAdderStimulus::new(i8::MAX, i8::MIN),
        SignedAdderStimulus::new(i8::MIN, 0),
        SignedAdderStimulus::new(0, i8::MIN),
    ]
    .into_iter()
}

/// Exhaustively enumerates every pair of signed eight-bit operands.
pub fn exhaustive_signed_adder_sequence() -> impl Iterator<Item = SignedAdderStimulus> {
    (i8::MIN..=i8::MAX)
        .flat_map(|lhs| (i8::MIN..=i8::MAX).map(move |rhs| SignedAdderStimulus::new(lhs, rhs)))
}

#[cfg(test)]
mod tests {
    use vvm::ReferenceModel;

    use super::{SignedAdderReferenceModel, SignedAdderStimulus, exhaustive_signed_adder_sequence};

    #[test]
    fn reference_model_preserves_full_signed_range() {
        let cases = [
            (SignedAdderStimulus::new(0, 0), 0),
            (SignedAdderStimulus::new(1, -1), 0),
            (SignedAdderStimulus::new(i8::MAX, i8::MAX), 254),
            (SignedAdderStimulus::new(i8::MIN, i8::MIN), -256),
            (SignedAdderStimulus::new(i8::MIN, i8::MAX), -1),
        ];

        let mut model = SignedAdderReferenceModel;

        for (stimulus, expected) in cases {
            let observation = model.predict(&stimulus);

            assert_eq!(observation.sum(), expected,);
        }
    }

    #[test]
    fn exhaustive_sequence_contains_all_input_pairs() {
        assert_eq!(exhaustive_signed_adder_sequence().count(), 65_536,);
    }
}
