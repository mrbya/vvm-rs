#![allow(
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::modulo_arithmetic
)]

use std::fmt;

/// Number of bits stored by one trasfer word.
const WORD_BITS: usize = 32;

/// Number of hexadecimal digits represented by one trasfer word.
const HEX_DIGITS_PER_WORD: usize = 8;

/// Unsigned packed bit value with exactly `N` significant bits.
///
/// Storage uses least-significant-word-first 32-bit words:
///
/// ```text
/// words[0] = bits 31:0
/// words[1] = bits 63:32
/// ...
/// ```
///
/// Unused bits in the final word are always zero.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Bits<const N: usize> {
    /// Canonical least-significant-word-first representation.
    words: Box<[u32]>,
}

/// Signed packed two's-complement value with exactly `N` significant bits.
///
/// Storage uses the same canonical bit representation as [`Bits`]. Signedness
/// affects interpretation only; unused bits in the final word remain zero
/// rather than being sign-extended into storage.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SignedBits<const N: usize> {
    /// Canonical least-significant-word-first representation.
    words: Box<[u32]>,
}

/// Error returned when a packed value is constructed from the wrong number of 32-bit words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBitVectorWordCount {
    /// Declared packed width.
    width: usize,

    /// Number of words required by the declared width.
    expected: usize,

    /// Number of words supplied by the caller.
    actual: usize,
}

impl InvalidBitVectorWordCount {
    /// Returns the declared packed width.
    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }

    /// Returns the required number of words.
    #[must_use]
    pub const fn expected(self) -> usize {
        self.expected
    }

    /// Returns the supplied number of words.
    #[must_use]
    pub const fn actual(self) -> usize {
        self.actual
    }
}

impl fmt::Display for InvalidBitVectorWordCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-bit value requires {} 32-bit words, but {} were supplied",
            self.width, self.expected, self.actual,
        )
    }
}

impl std::error::Error for InvalidBitVectorWordCount {}

impl<const N: usize> Bits<N> {
    /// Declared packed width.
    pub const WIDTH: usize = N;

    /// Number of 32-bit words required to store the declared wwidth.
    pub const WORDS: usize = word_count(N);

    /// Constructs an all-zero value.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            words: zero_words::<N>(),
        }
    }

    /// Constructs a value from least-significant-word-first 32-bit words.
    ///
    /// Unused bits in the final word are cleared.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidBitVectorWordCount`] when the supplied word count does not match [`Self::WORDS`].
    pub fn from_words_le(words: impl AsRef<[u32]>) -> Result<Self, InvalidBitVectorWordCount> {
        Ok(Self {
            words: canonical_words::<N>(words.as_ref())?,
        })
    }

    /// Returns the canonical least-significant-word-first representation.
    #[must_use]
    pub fn words_le(&self) -> &[u32] {
        &self.words
    }

    /// Returns one declared bit.
    ///
    /// Bit 0 is the least-significant bit.
    #[must_use]
    pub fn bit(&self, index: usize) -> Option<bool> {
        bit(&self.words, N, index)
    }

    /// Returns whether every declared bit is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|word| *word == 0)
    }
}

impl<const N: usize> SignedBits<N> {
    /// Declared packed width.
    pub const WIDTH: usize = N;

    /// Number of 32-bit words required to store the declared width.
    pub const WORDS: usize = word_count(N);

    /// Constructs an all-zero value.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            words: zero_words::<N>(),
        }
    }

    /// Constructs a signed value from least-significant-word-first words.
    ///
    /// The supplied words are interpreted as an `N`-bit two's-complement bit
    /// pattern. Unused bits in the final word are cleared.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidBitVectorWordCount`] when the supplied word count does not match [`Self::WORDS`].
    pub fn from_words_le(words: impl AsRef<[u32]>) -> Result<Self, InvalidBitVectorWordCount> {
        Ok(Self {
            words: canonical_words::<N>(words.as_ref())?,
        })
    }

    /// Returns the canonical least-significant-word-first representation.
    #[must_use]
    pub fn words_le(&self) -> &[u32] {
        &self.words
    }

    /// Returns one declared bit.
    ///
    /// Bit 0 is the least-significant bit.
    #[must_use]
    pub fn bit(&self, index: usize) -> Option<bool> {
        bit(&self.words, N, index)
    }

    /// Returns whether every declared bit is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|word| *word == 0)
    }

    /// Returns whether the declared sign bit is set.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        N.checked_sub(1)
            .and_then(|index| self.bit(index))
            .unwrap_or(false)
    }
}

impl<const N: usize> Default for Bits<N> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> Default for SignedBits<N> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> From<u64> for Bits<N> {
    fn from(value: u64) -> Self {
        Self {
            words: words_from_64::<N>(value.to_le_bytes(), 0),
        }
    }
}

impl<const N: usize> From<i64> for SignedBits<N> {
    fn from(value: i64) -> Self {
        let extension = if value.is_negative() { u32::MAX } else { 0 };

        Self {
            words: words_from_64::<N>(value.to_le_bytes(), extension),
        }
    }
}

impl<const N: usize> fmt::Debug for Bits<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_words(formatter, "Bits", N, &self.words)
    }
}

impl<const N: usize> fmt::Debug for SignedBits<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_words(formatter, "SignedBits", N, &self.words)
    }
}

/// Returns the number of 32-bit words needed for a width.
const fn word_count(width: usize) -> usize {
    let complete_words = width / WORD_BITS;
    let partial_bits = width % WORD_BITS;

    if partial_bits == 0 {
        complete_words
    } else {
        complete_words.saturating_add(1)
    }
}

/// Returns the number of hexadecimal digits needed for a width.
const fn hex_digit_count(width: usize) -> usize {
    let complete_digits = width / 4;
    let partial_bits = width % 4;

    if partial_bits == 0 {
        complete_digits
    } else {
        complete_digits.saturating_add(1)
    }
}

/// Returns the valid-bit mask for the final word.
const fn final_word_mask(width: usize) -> u32 {
    let final_bits = width % WORD_BITS;

    if final_bits == 0 {
        u32::MAX
    } else {
        u32::MAX >> (WORD_BITS - final_bits)
    }
}

/// Allocates an all-zero canonical representation.
fn zero_words<const N: usize>() -> Box<[u32]> {
    vec![0; word_count(N)].into_boxed_slice()
}

/// Validates, copies, and canonicalizes external words.
fn canonical_words<const N: usize>(words: &[u32]) -> Result<Box<[u32]>, InvalidBitVectorWordCount> {
    let expected = word_count(N);
    let actual = words.len();

    if actual != expected {
        return Err(InvalidBitVectorWordCount {
            width: N,
            expected,
            actual,
        });
    }

    let mut canonical = words.to_vec().into_boxed_slice();

    mask_final_word::<N>(&mut canonical);

    Ok(canonical)
}

/// Clears unused bits in the final word.
fn mask_final_word<const N: usize>(words: &mut [u32]) {
    let Some(final_word) = words.last_mut() else {
        return;
    };

    *final_word &= final_word_mask(N);
}

/// Constructs canonical words from one 64-bit bit pattern.
///
/// `extension` is used for words above the low 64 bits and allows signed
/// construction to sign-extend before the final declared-width mask.
fn words_from_64<const N: usize>(bytes: [u8; 8], extension: u32) -> Box<[u32]> {
    let [
        byte_0,
        byte_1,
        byte_2,
        byte_3,
        byte_4,
        byte_5,
        byte_6,
        byte_7,
    ] = bytes;

    let mut words = vec![extension; word_count(N)].into_boxed_slice();

    if let Some(low_word) = words.first_mut() {
        *low_word = u32::from_le_bytes([byte_0, byte_1, byte_2, byte_3]);
    }

    if let Some(high_word) = words.get_mut(1) {
        *high_word = u32::from_le_bytes([byte_4, byte_5, byte_6, byte_7]);
    }

    mask_final_word::<N>(&mut words);

    words
}

/// Reads one bit from canonical word storage.
fn bit(words: &[u32], width: usize, index: usize) -> Option<bool> {
    if index >= width {
        return None;
    }

    let word_index = index / WORD_BITS;
    let bit_index = index % WORD_BITS;

    words
        .get(word_index)
        .map(|word| word & (1_u32 << bit_index) != 0)
}

/// Formats a canonical packed value as fixed-width hexadecimal.
fn format_words(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
    width: usize,
    words: &[u32],
) -> fmt::Result {
    write!(formatter, "{name}<{width}>(0x")?;

    if words.is_empty() {
        formatter.write_str("0")?;
        return formatter.write_str(")");
    }

    let total_digits = hex_digit_count(width);

    let trailing_digits = words
        .len()
        .saturating_sub(1)
        .saturating_mul(HEX_DIGITS_PER_WORD);

    let leading_digits = total_digits.saturating_sub(trailing_digits);

    for (index, word) in words.iter().rev().enumerate() {
        let digits = if index == 0 {
            leading_digits
        } else {
            HEX_DIGITS_PER_WORD
        };

        write!(formatter, "{word:0digits$x}")?;
    }

    formatter.write_str(")")
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::{Bits, InvalidBitVectorWordCount, SignedBits};

    #[test]
    fn calculates_word_counts_at_boundaries() {
        assert_eq!(Bits::<1>::WORDS, 1);
        assert_eq!(Bits::<31>::WORDS, 1);
        assert_eq!(Bits::<32>::WORDS, 1);
        assert_eq!(Bits::<33>::WORDS, 2);
        assert_eq!(Bits::<64>::WORDS, 2);
        assert_eq!(Bits::<65>::WORDS, 3);
        assert_eq!(Bits::<96>::WORDS, 3);
        assert_eq!(Bits::<129>::WORDS, 5);
        assert_eq!(Bits::<256>::WORDS, 8);

        assert_eq!(SignedBits::<65>::WORDS, 3);
        assert_eq!(SignedBits::<129>::WORDS, 5);
    }

    #[test]
    fn rejects_incorrect_word_count() {
        let error = Bits::<65>::from_words_le([0, 0]);

        assert_eq!(
            error,
            Err(InvalidBitVectorWordCount {
                width: 65,
                expected: 3,
                actual: 2,
            }),
        );
    }

    #[test]
    fn reports_word_count_error_details() {
        let error = Bits::<129>::from_words_le([0]).expect_err("incorrect word count must fail");

        assert_eq!(error.width(), 129);
        assert_eq!(error.expected(), 5);
        assert_eq!(error.actual(), 1);

        assert_eq!(
            error.to_string(),
            "129-bit value requires 5 32-bit words, but 1 were supplied",
        );
    }

    #[test]
    fn masks_unused_unsigned_top_bits() {
        let value = Bits::<65>::from_words_le([0x0123_4567, 0x89ab_cdef, u32::MAX])
            .expect("correct word count must succeed");

        assert_eq!(value.words_le(), &[0x0123_4567, 0x89ab_cdef, 0x0000_0001,],);
    }

    #[test]
    fn masks_unused_signed_top_bits() {
        let value =
            SignedBits::<129>::from_words_le([u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX])
                .expect("correct word count must succeed");

        assert_eq!(
            value.words_le(),
            &[u32::MAX, u32::MAX, u32::MAX, u32::MAX, 0x0000_0001,],
        );

        assert!(value.is_negative());
    }

    #[test]
    fn constructs_unsigned_values_from_u64() {
        let value = Bits::<65>::from(0x0123_4567_89ab_cdef);

        assert_eq!(value.words_le(), &[0x89ab_cdef, 0x0123_4567, 0,],);
    }

    #[test]
    fn truncates_unsigned_values_to_declared_width() {
        let value = Bits::<33>::from(u64::MAX);

        assert_eq!(value.words_le(), &[u32::MAX, 1],);
    }

    #[test]
    fn sign_extends_negative_i64_values() {
        let value = SignedBits::<129>::from(-1_i64);

        assert_eq!(
            value.words_le(),
            &[u32::MAX, u32::MAX, u32::MAX, u32::MAX, 1,],
        );

        assert!(value.is_negative());
    }

    #[test]
    fn zero_extends_positive_i64_values() {
        let value = SignedBits::<129>::from(1_i64);

        assert_eq!(value.words_le(), &[1, 0, 0, 0, 0],);

        assert!(!value.is_negative());
    }

    #[test]
    fn truncates_negative_values_to_declared_width() {
        let value = SignedBits::<5>::from(-1_i64);

        assert_eq!(value.words_le(), &[0x1f]);
        assert!(value.is_negative());
    }

    #[test]
    fn reads_declared_bits() {
        let value = Bits::<65>::from_words_le([1, 0, 1]).expect("correct word count must succeed");

        assert_eq!(value.bit(0), Some(true));
        assert_eq!(value.bit(1), Some(false));
        assert_eq!(value.bit(64), Some(true));
        assert_eq!(value.bit(65), None);
    }

    #[test]
    fn identifies_zero_values() {
        assert!(Bits::<256>::zero().is_zero());
        assert!(SignedBits::<129>::zero().is_zero());

        assert!(!Bits::<65>::from(1_u64).is_zero());
    }

    #[test]
    fn formats_full_declared_width() {
        let unsigned = Bits::<65>::from_words_le([0x89ab_cdef, 0x0123_4567, 1])
            .expect("correct word count must succeed");

        let signed = SignedBits::<65>::from(-1_i64);

        assert_eq!(format!("{unsigned:?}"), "Bits<65>(0x10123456789abcdef)");

        assert_eq!(format!("{signed:?}"), "SignedBits<65>(0x1ffffffffffffffff)");
    }

    #[test]
    fn canonical_values_have_stable_equality_and_hashing() {
        let left =
            Bits::<65>::from_words_le([1, 2, u32::MAX]).expect("correct word count must succeed");

        let right = Bits::<65>::from_words_le([1, 2, 1]).expect("correct word count must succeed");

        assert_eq!(left, right);

        let mut left_hasher = DefaultHasher::new();

        let mut right_hasher = DefaultHasher::new();

        left.hash(&mut left_hasher);
        right.hash(&mut right_hasher);

        assert_eq!(left_hasher.finish(), right_hasher.finish(),);
    }
}
