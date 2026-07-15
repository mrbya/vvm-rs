#![allow(
    clippy::arithmetic_side_effects,
    clippy::integer_division,
    clippy::modulo_arithmetic
)]

use std::fmt;

use crate::{Bits, InvalidBitVectorWordCount, SignedBits};

/// Number of bits in one canonical packed storage word.
const WORD_BITS: usize = 32;

/// Maximum scalar field width that can be extracted into `u64` or `i64`.
const MAX_SCALAR_BITS: usize = 64;

/// Packed value that can be represented by canonical 32-bit little-endian words.
pub trait PackedValue: Sized {
    /// Declared packed width.
    const WIDTH: usize;

    /// Number of canonical 32-bit words.
    const WORDS: usize;

    /// Constructs the value from canonical least-significant-word-first words.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidBitVectorWordCount`] if the supplied word count does not
    /// match the declared width.
    fn from_words_le(words: impl AsRef<[u32]>) -> Result<Self, InvalidBitVectorWordCount>;

    /// Returns canonical least-significant-word-first words.
    fn words_le(&self) -> &[u32];
}

impl<const N: usize> PackedValue for Bits<N> {
    const WIDTH: usize = Self::WIDTH;
    const WORDS: usize = Self::WORDS;

    fn from_words_le(words: impl AsRef<[u32]>) -> Result<Self, InvalidBitVectorWordCount> {
        Self::from_words_le(words)
    }

    fn words_le(&self) -> &[u32] {
        self.words_le()
    }
}

impl<const N: usize> PackedValue for SignedBits<N> {
    const WIDTH: usize = Self::WIDTH;
    const WORDS: usize = Self::WORDS;

    fn from_words_le(words: impl AsRef<[u32]>) -> Result<Self, InvalidBitVectorWordCount> {
        Self::from_words_le(words)
    }

    fn words_le(&self) -> &[u32] {
        self.words_le()
    }
}

/// One packed field layout inside a packed aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedFieldLayout {
    /// Field name.
    name: &'static str,

    /// Bit offset from the least-significant bit of the containing aggregate.
    offset: usize,

    /// Field width in bits.
    width: usize,

    /// Whether the field is signed.
    signed: bool,
}

impl PackedFieldLayout {
    /// Creates one packed field layout.
    #[must_use]
    pub const fn new(name: &'static str, offset: usize, width: usize, signed: bool) -> Self {
        Self {
            name,
            offset,
            width,
            signed,
        }
    }

    /// Returns the field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the field offset from the aggregate LSB.
    #[must_use]
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns the field width.
    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }

    /// Returns whether the field is signed.
    #[must_use]
    pub const fn signed(self) -> bool {
        self.signed
    }
}

/// Packed aggregate layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedLayout {
    /// Aggregate name.
    name: &'static str,

    /// Aggregate width in bits.
    width: usize,

    /// Declared fields.
    fields: &'static [PackedFieldLayout],
}

impl PackedLayout {
    /// Creates one packed aggregate layout.
    #[must_use]
    pub const fn new(
        name: &'static str,
        width: usize,
        fields: &'static [PackedFieldLayout],
    ) -> Self {
        Self {
            name,
            width,
            fields,
        }
    }

    /// Returns the aggregate name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the aggregate width.
    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }

    /// Returns the aggregate fields.
    #[must_use]
    pub const fn fields(self) -> &'static [PackedFieldLayout] {
        self.fields
    }
}

/// Validated packed bit range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedRange {
    /// Total containing storage width.
    storage_width: usize,

    /// Bit offset from LSB.
    offset: usize,

    /// Field width.
    width: usize,
}

impl PackedRange {
    /// Creates a validated packed range.
    ///
    /// # Errors
    ///
    /// Returns [`PackedLayoutError`] when the range is empty or outside the
    /// containing storage width.
    pub fn new(
        storage_width: usize,
        offset: usize,
        width: usize,
    ) -> Result<Self, PackedLayoutError> {
        validate_range(storage_width, offset, width)?;

        Ok(Self {
            storage_width,
            offset,
            width,
        })
    }

    /// Returns the containing storage width.
    #[must_use]
    pub const fn storage_width(self) -> usize {
        self.storage_width
    }

    /// Returns the range offset.
    #[must_use]
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns the range width.
    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }
}

/// Error returned by packed-layout extraction and insertion helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedLayoutError {
    /// A packed field width was zero.
    ZeroWidth {
        /// Containing packed storage width.
        storage_width: usize,

        /// Requested field offset.
        offset: usize,
    },

    /// A requested packed field is outside the containing storage.
    RangeOutOfBounds {
        /// Containing packed storage width.
        storage_width: usize,

        /// Requested field offset.
        offset: usize,

        /// Requested field width.
        width: usize,
    },

    /// A scalar extraction or insertion requested more than 64 bits.
    ScalarFieldTooWide {
        /// Requested field width.
        width: usize,

        /// Maximum scalar field width.
        maximum: usize,
    },

    /// The supplied word slice length does not match the containing storage width.
    InvalidWordCount {
        /// Containing packed storage width.
        storage_width: usize,

        /// Expected canonical word count.
        expected: usize,

        /// Actual supplied word count.
        actual: usize,
    },

    /// A packed value could not be reconstructed from extracted words.
    InvalidPackedValue {
        /// Underlying bit-vector construction error.
        source: InvalidBitVectorWordCount,
    },
}

impl fmt::Display for PackedLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ZeroWidth {
                storage_width,
                offset,
            } => write!(
                formatter,
                "packed field at offset {offset} in {storage_width}-bit storage has zero width",
            ),
            Self::RangeOutOfBounds {
                storage_width,
                offset,
                width,
            } => write!(
                formatter,
                "packed field offset {offset} width {width} is outside {storage_width}-bit storage",
            ),
            Self::ScalarFieldTooWide { width, maximum } => write!(
                formatter,
                "packed scalar field is {width} bits wide; maximum scalar width is {maximum}",
            ),
            Self::InvalidWordCount {
                storage_width,
                expected,
                actual,
            } => write!(
                formatter,
                "{storage_width}-bit packed storage requires {expected} 32-bit words, but \
                 {actual} were supplied",
            ),
            Self::InvalidPackedValue { ref source } => {
                write!(
                    formatter,
                    "failed to construct extracted packed value: {source}"
                )
            }
        }
    }
}

impl std::error::Error for PackedLayoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::InvalidPackedValue { ref source } => Some(source),
            Self::ZeroWidth { .. }
            | Self::RangeOutOfBounds { .. }
            | Self::ScalarFieldTooWide { .. }
            | Self::InvalidWordCount { .. } => None,
        }
    }
}

/// Returns the expected canonical word count for a packed width.
const fn word_count(width: usize) -> usize {
    let complete_words = width / WORD_BITS;
    let partial_bits = width % WORD_BITS;

    if partial_bits == 0 {
        complete_words
    } else {
        complete_words.saturating_add(1)
    }
}

/// Validates that a word slice matches the containing storage width.
const fn validate_words(words: &[u32], storage_width: usize) -> Result<(), PackedLayoutError> {
    let expected = word_count(storage_width);
    let actual = words.len();

    if actual != expected {
        return Err(PackedLayoutError::InvalidWordCount {
            storage_width,
            expected,
            actual,
        });
    }

    Ok(())
}

/// Validates a field range.
fn validate_range(
    storage_width: usize,
    offset: usize,
    width: usize,
) -> Result<(), PackedLayoutError> {
    if width == 0 {
        return Err(PackedLayoutError::ZeroWidth {
            storage_width,
            offset,
        });
    }

    let end = offset
        .checked_add(width)
        .ok_or(PackedLayoutError::RangeOutOfBounds {
            storage_width,
            offset,
            width,
        })?;

    if end > storage_width {
        return Err(PackedLayoutError::RangeOutOfBounds {
            storage_width,
            offset,
            width,
        });
    }

    Ok(())
}

/// Validates a scalar field width.
const fn validate_scalar_width(width: usize) -> Result<(), PackedLayoutError> {
    if width > MAX_SCALAR_BITS {
        return Err(PackedLayoutError::ScalarFieldTooWide {
            width,
            maximum: MAX_SCALAR_BITS,
        });
    }

    Ok(())
}

/// Reads one bit from canonical packed storage.
fn read_bit(words: &[u32], storage_width: usize, index: usize) -> Result<bool, PackedLayoutError> {
    validate_range(storage_width, index, 1)?;
    validate_words(words, storage_width)?;

    let word_index = index / WORD_BITS;
    let bit_index = index % WORD_BITS;

    let Some(word) = words.get(word_index) else {
        return Err(PackedLayoutError::InvalidWordCount {
            storage_width,
            expected: word_count(storage_width),
            actual: words.len(),
        });
    };

    let shift = u32::try_from(bit_index).map_err(|_error| PackedLayoutError::RangeOutOfBounds {
        storage_width,
        offset: index,
        width: 1,
    })?;

    let Some(mask) = 1_u32.checked_shl(shift) else {
        return Err(PackedLayoutError::RangeOutOfBounds {
            storage_width,
            offset: index,
            width: 1,
        });
    };

    Ok(word & mask != 0)
}

/// Writes one bit into canonical packed storage.
fn write_bit(
    words: &mut [u32],
    storage_width: usize,
    index: usize,
    value: bool,
) -> Result<(), PackedLayoutError> {
    validate_range(storage_width, index, 1)?;

    let expected = word_count(storage_width);
    let actual = words.len();

    if actual != expected {
        return Err(PackedLayoutError::InvalidWordCount {
            storage_width,
            expected,
            actual,
        });
    }

    let word_index = index / WORD_BITS;
    let bit_index = index % WORD_BITS;

    let Some(word) = words.get_mut(word_index) else {
        return Err(PackedLayoutError::InvalidWordCount {
            storage_width,
            expected,
            actual,
        });
    };

    let shift = u32::try_from(bit_index).map_err(|_error| PackedLayoutError::RangeOutOfBounds {
        storage_width,
        offset: index,
        width: 1,
    })?;

    let Some(mask) = 1_u32.checked_shl(shift) else {
        return Err(PackedLayoutError::RangeOutOfBounds {
            storage_width,
            offset: index,
            width: 1,
        });
    };

    if value {
        *word |= mask;
    } else {
        *word &= !mask;
    }

    Ok(())
}

/// Extracts an unsigned scalar field from canonical packed storage.
///
/// `offset` is measured from the least-significant bit of `words`.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] when the field range is invalid, the word count
/// is invalid, or the field is wider than 64 bits.
pub fn extract_unsigned(
    words: &[u32],
    storage_width: usize,
    offset: usize,
    width: usize,
) -> Result<u64, PackedLayoutError> {
    validate_words(words, storage_width)?;
    validate_range(storage_width, offset, width)?;
    validate_scalar_width(width)?;

    let mut value = 0_u64;

    for bit_offset in 0..width {
        let source_index =
            offset
                .checked_add(bit_offset)
                .ok_or(PackedLayoutError::RangeOutOfBounds {
                    storage_width,
                    offset,
                    width,
                })?;

        if read_bit(words, storage_width, source_index)? {
            let shift = u32::try_from(bit_offset).map_err(|_error| {
                PackedLayoutError::RangeOutOfBounds {
                    storage_width,
                    offset,
                    width,
                }
            })?;

            if let Some(mask) = 1_u64.checked_shl(shift) {
                value |= mask;
            }
        }
    }

    Ok(value)
}

/// Inserts an unsigned scalar field into canonical packed storage.
///
/// Values wider than the destination field are truncated to the field width.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] when the field range is invalid, the word count
/// is invalid, or the field is wider than 64 bits.
pub fn insert_unsigned(
    words: &mut [u32],
    storage_width: usize,
    offset: usize,
    width: usize,
    value: u64,
) -> Result<(), PackedLayoutError> {
    validate_range(storage_width, offset, width)?;
    validate_scalar_width(width)?;

    for bit_offset in 0..width {
        let shift =
            u32::try_from(bit_offset).map_err(|_error| PackedLayoutError::RangeOutOfBounds {
                storage_width,
                offset,
                width,
            })?;

        let bit = value
            .checked_shr(shift)
            .is_some_and(|shifted| shifted & 1 != 0);

        let target_index =
            offset
                .checked_add(bit_offset)
                .ok_or(PackedLayoutError::RangeOutOfBounds {
                    storage_width,
                    offset,
                    width,
                })?;

        write_bit(words, storage_width, target_index, bit)?;
    }

    Ok(())
}

/// Extracts a signed scalar field with explicit sign extension.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] when the field range is invalid, the word count
/// is invalid, or the field is wider than 64 bits.
pub fn extract_signed(
    words: &[u32],
    storage_width: usize,
    offset: usize,
    width: usize,
) -> Result<i64, PackedLayoutError> {
    let raw = extract_unsigned(words, storage_width, offset, width)?;

    let sign_bit_index = width.checked_sub(1).ok_or(PackedLayoutError::ZeroWidth {
        storage_width,
        offset,
    })?;

    let sign_bit_shift =
        u32::try_from(sign_bit_index).map_err(|_error| PackedLayoutError::RangeOutOfBounds {
            storage_width,
            offset,
            width,
        })?;

    let Some(sign_bit) = 1_u64.checked_shl(sign_bit_shift) else {
        return Err(PackedLayoutError::ScalarFieldTooWide {
            width,
            maximum: MAX_SCALAR_BITS,
        });
    };

    if raw & sign_bit == 0 {
        return i64::try_from(raw).map_err(|_error| PackedLayoutError::ScalarFieldTooWide {
            width,
            maximum: MAX_SCALAR_BITS,
        });
    }

    let extended = if width == MAX_SCALAR_BITS {
        raw
    } else {
        let shift =
            u32::try_from(width).map_err(|_error| PackedLayoutError::ScalarFieldTooWide {
                width,
                maximum: MAX_SCALAR_BITS,
            })?;

        let Some(extension_mask) = u64::MAX.checked_shl(shift) else {
            return Err(PackedLayoutError::ScalarFieldTooWide {
                width,
                maximum: MAX_SCALAR_BITS,
            });
        };

        raw | extension_mask
    };

    Ok(i64::from_ne_bytes(extended.to_ne_bytes()))
}

/// Inserts a signed scalar field using its two's-complement low bits.
///
/// Values wider than the destination field are truncated to the field width.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] when the field range is invalid, the word count
/// is invalid, or the field is wider than 64 bits.
pub fn insert_signed(
    words: &mut [u32],
    storage_width: usize,
    offset: usize,
    width: usize,
    value: i64,
) -> Result<(), PackedLayoutError> {
    let raw = u64::from_ne_bytes(value.to_ne_bytes());

    insert_unsigned(words, storage_width, offset, width, raw)
}

/// Extracts a packed subfield from canonical packed storage.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] if the field range, word count, or reconstructed
/// packed value is invalid.
pub fn extract_packed<P>(
    words: &[u32],
    storage_width: usize,
    offset: usize,
) -> Result<P, PackedLayoutError>
where
    P: PackedValue,
{
    validate_words(words, storage_width)?;
    validate_range(storage_width, offset, P::WIDTH)?;

    let mut output = vec![0_u32; P::WORDS];

    for bit_offset in 0..P::WIDTH {
        let source_index =
            offset
                .checked_add(bit_offset)
                .ok_or(PackedLayoutError::RangeOutOfBounds {
                    storage_width,
                    offset,
                    width: P::WIDTH,
                })?;

        let bit = read_bit(words, storage_width, source_index)?;

        write_bit(&mut output, P::WIDTH, bit_offset, bit)?;
    }

    P::from_words_le(output).map_err(|source| PackedLayoutError::InvalidPackedValue { source })
}

/// Inserts a packed subfield into canonical packed storage.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] if the field range or word count is invalid.
pub fn insert_packed<P>(
    words: &mut [u32],
    storage_width: usize,
    offset: usize,
    value: &P,
) -> Result<(), PackedLayoutError>
where
    P: PackedValue,
{
    validate_range(storage_width, offset, P::WIDTH)?;

    for bit_offset in 0..P::WIDTH {
        let bit = read_bit(value.words_le(), P::WIDTH, bit_offset)?;

        let target_index =
            offset
                .checked_add(bit_offset)
                .ok_or(PackedLayoutError::RangeOutOfBounds {
                    storage_width,
                    offset,
                    width: P::WIDTH,
                })?;

        write_bit(words, storage_width, target_index, bit)?;
    }

    Ok(())
}

/// Extracts a signed packed subfield.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] if the field range, word count, or reconstructed
/// packed value is invalid.
pub fn extract_signed_packed<const N: usize>(
    words: &[u32],
    storage_width: usize,
    offset: usize,
) -> Result<SignedBits<N>, PackedLayoutError> {
    extract_packed(words, storage_width, offset)
}

/// Inserts a signed packed subfield.
///
/// # Errors
///
/// Returns [`PackedLayoutError`] if the field range or word count is invalid.
pub fn insert_signed_packed<const N: usize>(
    words: &mut [u32],
    storage_width: usize,
    offset: usize,
    value: &SignedBits<N>,
) -> Result<(), PackedLayoutError> {
    insert_packed(words, storage_width, offset, value)
}

#[cfg(test)]
mod tests {
    use super::{
        PackedFieldLayout, PackedLayout, PackedLayoutError, PackedRange, extract_packed,
        extract_signed, extract_unsigned, insert_packed, insert_signed, insert_unsigned,
    };
    use crate::{Bits, SignedBits};

    #[test]
    fn rejects_zero_width_range() {
        assert!(matches!(
            PackedRange::new(16, 0, 0),
            Err(PackedLayoutError::ZeroWidth {
                storage_width: 16,
                offset: 0,
            }),
        ));
    }

    #[test]
    fn rejects_out_of_bounds_range() {
        assert!(matches!(
            PackedRange::new(16, 12, 5),
            Err(PackedLayoutError::RangeOutOfBounds {
                storage_width: 16,
                offset: 12,
                width: 5,
            }),
        ));
    }

    #[test]
    fn rejects_invalid_word_count() {
        assert!(matches!(
            extract_unsigned(&[0], 65, 0, 1),
            Err(PackedLayoutError::InvalidWordCount {
                storage_width: 65,
                expected: 3,
                actual: 1,
            }),
        ));
    }

    #[test]
    fn rejects_scalar_field_wider_than_sixty_four_bits() {
        assert!(matches!(
            extract_unsigned(&[0, 0, 0], 65, 0, 65),
            Err(PackedLayoutError::ScalarFieldTooWide {
                width: 65,
                maximum: 64,
            }),
        ));
    }

    #[test]
    fn packed_field_layout_getters_work() {
        let field = PackedFieldLayout::new("opcode", 12, 4, false);

        assert_eq!(field.name(), "opcode");
        assert_eq!(field.offset(), 12);
        assert_eq!(field.width(), 4);
        assert!(!field.signed());
    }

    #[test]
    fn packed_layout_getters_work() {
        const FIELDS: [PackedFieldLayout; 2] = [
            PackedFieldLayout::new("low", 0, 8, false),
            PackedFieldLayout::new("high", 8, 8, false),
        ];

        let layout = PackedLayout::new("byte_pair", 16, &FIELDS);

        assert_eq!(layout.name(), "byte_pair");
        assert_eq!(layout.width(), 16);
        assert_eq!(layout.fields(), &FIELDS);
    }

    #[test]
    fn packed_range_getters_work() -> Result<(), Box<dyn std::error::Error>> {
        let range = PackedRange::new(16, 8, 4)?;

        assert_eq!(range.storage_width(), 16);
        assert_eq!(range.offset(), 8);
        assert_eq!(range.width(), 4);

        Ok(())
    }

    #[test]
    fn display_message_is_human_readable() {
        let error = PackedLayoutError::InvalidWordCount {
            storage_width: 65,
            expected: 3,
            actual: 1,
        };

        assert_eq!(
            error.to_string(),
            "65-bit packed storage requires 3 32-bit words, but 1 were supplied",
        );
    }

    #[test]
    fn invalid_packed_value_exposes_source() -> Result<(), Box<dyn std::error::Error>> {
        let Err(source) = Bits::<65>::from_words_le([0_u32, 0_u32]) else {
            return Err(
                std::io::Error::other("expected incorrect 65-bit word count to fail").into(),
            );
        };

        let error = PackedLayoutError::InvalidPackedValue { source };

        assert!(std::error::Error::source(&error).is_some());

        Ok(())
    }

    #[test]
    fn extracts_unsigned_fields_from_packed_struct_layout() -> Result<(), Box<dyn std::error::Error>>
    {
        let words = [0xabcd_u32];

        assert_eq!(extract_unsigned(&words, 16, 0, 8)?, 0xcd);
        assert_eq!(extract_unsigned(&words, 16, 8, 3)?, 0b011);
        assert_eq!(extract_unsigned(&words, 16, 11, 1)?, 1);
        assert_eq!(extract_unsigned(&words, 16, 12, 4)?, 0x0a);

        Ok(())
    }

    #[test]
    fn inserts_unsigned_fields_into_packed_struct_layout() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut words = [0_u32];

        insert_unsigned(&mut words, 16, 0, 8, 0xcd)?;
        insert_unsigned(&mut words, 16, 8, 3, 0b011)?;
        insert_unsigned(&mut words, 16, 11, 1, 1)?;
        insert_unsigned(&mut words, 16, 12, 4, 0x0a)?;

        assert_eq!(words, [0xabcd]);

        Ok(())
    }

    #[test]
    fn insert_unsigned_truncates_to_field_width() -> Result<(), Box<dyn std::error::Error>> {
        let mut words = [0_u32];

        insert_unsigned(&mut words, 8, 0, 4, 0xff)?;

        assert_eq!(words, [0x0f]);

        Ok(())
    }

    #[test]
    fn extracts_signed_fields_with_sign_extension() -> Result<(), Box<dyn std::error::Error>> {
        let negative_words = [0b1_1011_u32];

        assert_eq!(extract_signed(&negative_words, 5, 0, 5)?, -5);

        let positive_words = [0b0_1011_u32];

        assert_eq!(extract_signed(&positive_words, 5, 0, 5)?, 11);

        Ok(())
    }

    #[test]
    fn insert_signed_truncates_twos_complement_bits() -> Result<(), Box<dyn std::error::Error>> {
        let mut words = [0_u32];

        insert_signed(&mut words, 5, 0, 5, -1)?;

        assert_eq!(words, [0x1f]);

        Ok(())
    }

    #[test]
    fn extracts_field_across_word_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let words = [0x8000_0000, 0x0000_0001];

        assert_eq!(extract_unsigned(&words, 64, 31, 2)?, 0b11);

        Ok(())
    }

    #[test]
    fn inserts_field_across_word_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let mut words = [0_u32, 0_u32];

        insert_unsigned(&mut words, 64, 31, 2, 0b11)?;

        assert_eq!(words, [0x8000_0000, 0x0000_0001]);

        Ok(())
    }

    #[test]
    fn extracts_packed_subfield() -> Result<(), Box<dyn std::error::Error>> {
        let words = [0x0123_4567, 0x89ab_cdef, 0x0000_0001];

        let value: Bits<65> = extract_packed(&words, 96, 0)?;

        assert_eq!(value.words_le(), &[0x0123_4567, 0x89ab_cdef, 0x0000_0001],);

        Ok(())
    }

    #[test]
    fn inserts_packed_subfield() -> Result<(), Box<dyn std::error::Error>> {
        let value = Bits::<65>::from_words_le([0x0123_4567, 0x89ab_cdef, 0x0000_0001])?;

        let mut words = [0_u32, 0_u32, 0_u32, 0_u32];

        insert_packed(&mut words, 128, 32, &value)?;

        assert_eq!(words, [0x0000_0000, 0x0123_4567, 0x89ab_cdef, 0x0000_0001],);

        Ok(())
    }

    #[test]
    fn extracts_signed_packed_subfield() -> Result<(), Box<dyn std::error::Error>> {
        let words = [u32::MAX, u32::MAX, 0x0000_0001];

        let value: SignedBits<65> = extract_packed(&words, 96, 0)?;

        assert!(value.is_negative());
        assert_eq!(value.words_le(), &[u32::MAX, u32::MAX, 0x0000_0001]);

        Ok(())
    }
}
