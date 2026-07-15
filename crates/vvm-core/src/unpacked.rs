//! Unpacked-array indexing helpers.

use std::fmt;

/// Error returned when an HDL unpacked-array index is outside its declared range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnpackedArrayIndexError {
    /// Requested HDL index.
    index: i64,
    /// Declared left HDL bound.
    left: i64,
    /// Declared right HDL bound.
    right: i64,
}

impl UnpackedArrayIndexError {
    /// Creates an unpacked-array index error.
    #[must_use]
    pub const fn new(index: i64, left: i64, right: i64) -> Self {
        Self { index, left, right }
    }

    /// Returns the invalid HDL index.
    #[must_use]
    pub const fn index(self) -> i64 {
        self.index
    }
    /// Returns the declared left bound.
    #[must_use]
    pub const fn left(self) -> i64 {
        self.left
    }
    /// Returns the declared right bound.
    #[must_use]
    pub const fn right(self) -> i64 {
        self.right
    }
}

impl fmt::Display for UnpackedArrayIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HDL unpacked-array index {} is outside [{}:{}]",
            self.index, self.left, self.right
        )
    }
}

impl std::error::Error for UnpackedArrayIndexError {}

/// Converts an HDL unpacked-array index into a zero-based declaration-order ordinal.
///
/// # Errors
///
/// Returns [`UnpackedArrayIndexError`] when `index` is outside the declared range or
/// the ordinal cannot be represented as `usize`.
pub fn unpacked_array_ordinal(
    left: i64,
    right: i64,
    index: i64,
) -> Result<usize, UnpackedArrayIndexError> {
    let error = UnpackedArrayIndexError::new(index, left, right);
    let in_range = if left <= right {
        index >= left && index <= right
    } else {
        index <= left && index >= right
    };
    if !in_range {
        return Err(error);
    }
    let ordinal = if left <= right {
        index.checked_sub(left)
    } else {
        left.checked_sub(index)
    };
    let Some(ordinal) = ordinal else {
        return Err(error);
    };
    usize::try_from(ordinal).map_err(|_conversion_error| error)
}

#[cfg(test)]
mod tests {
    use super::{UnpackedArrayIndexError, unpacked_array_ordinal};

    #[test]
    fn maps_declaration_order_for_both_directions() {
        assert_eq!(unpacked_array_ordinal(0, 3, 0), Ok(0));
        assert_eq!(unpacked_array_ordinal(0, 3, 3), Ok(3));
        assert_eq!(unpacked_array_ordinal(3, 0, 3), Ok(0));
        assert_eq!(unpacked_array_ordinal(3, 0, 0), Ok(3));
        assert_eq!(unpacked_array_ordinal(-2, 1, -1), Ok(1));
        assert_eq!(unpacked_array_ordinal(2, -1, 0), Ok(2));
    }

    #[test]
    fn rejects_out_of_range_and_preserves_context() {
        let error = unpacked_array_ordinal(2, 0, 3).expect_err("index is out of range");
        assert_eq!(error, UnpackedArrayIndexError::new(3, 2, 0));
        assert_eq!(error.index(), 3);
        assert_eq!(error.left(), 2);
        assert_eq!(error.right(), 0);
        assert_eq!(
            error.to_string(),
            "HDL unpacked-array index 3 is outside [2:0]"
        );
    }

    #[test]
    fn handles_i64_boundaries() {
        assert_eq!(unpacked_array_ordinal(i64::MIN, i64::MIN, i64::MIN), Ok(0));
        assert_eq!(unpacked_array_ordinal(i64::MAX, i64::MAX, i64::MAX), Ok(0));
    }
}
