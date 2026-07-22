use std::cmp::Ordering;

/// Type-erased equality operation for one concrete value type.
type EqualityFunction<T> = fn(&T, &T) -> bool;

/// Type-erased partial comparison operation for one concrete value type.
type ComparisonFunction<T> = fn(&T, &T) -> Option<std::cmp::Ordering>;

/// Declarative matcher owned by bin definition.
#[derive(Clone)]
pub enum BinMatcher<T> {
    /// Matches one exact value.
    Value {
        /// Expected value.
        expected: T,

        /// Monomorphized equality operation.
        equals: EqualityFunction<T>,
    },

    /// Matches any value from one owned set.
    Values {
        /// Expected values in declaration order.
        expected: Box<[T]>,

        /// Monomorphized equality operation.
        equals: EqualityFunction<T>,
    },

    /// Matches one inclusive range.
    InclusiveRange {
        /// Inclusive lower bound.
        start: T,

        /// Inclusive upper bound.
        end: T,

        /// Monomorphized partial comparison operation.
        compare: ComparisonFunction<T>,
    },
}

impl<T> BinMatcher<T> {
    /// Creates an exact-value matcher.
    pub fn value(expected: T) -> Self
    where
        T: PartialEq,
    {
        Self::Value {
            expected,
            equals: values_equal::<T>,
        }
    }

    /// Creates a value-set matcher.
    pub fn values(expected: impl IntoIterator<Item = T>) -> Self
    where
        T: PartialEq,
    {
        Self::Values {
            expected: expected.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            equals: values_equal::<T>,
        }
    }

    /// Creates an inclusive-range matcher.
    pub fn inclusive_range(start: T, end: T) -> Self
    where
        T: PartialOrd,
    {
        Self::InclusiveRange {
            start,
            end,
            compare: compare_values::<T>,
        }
    }

    /// Returns whether this matcher accepts a sampled value.
    pub fn matches(&self, sampled: &T) -> bool {
        match *self {
            Self::Value {
                ref expected,
                equals,
            } => equals(sampled, expected),

            Self::Values {
                ref expected,
                equals,
            } => expected.iter().any(|candidate| equals(sampled, candidate)),

            Self::InclusiveRange {
                ref start,
                ref end,
                compare,
            } => {
                let lower = compare(sampled, start);
                let upper = compare(sampled, end);

                matches!(lower, Some(Ordering::Equal | Ordering::Greater))
                    && matches!(upper, Some(Ordering::Equal | Ordering::Less))
            }
        }
    }

    /// Validates the matcher definition.
    pub fn validate(&self) -> Result<(), MatcherValidationError> {
        match *self {
            Self::Value { .. } => Ok(()),

            Self::Values {
                ref expected,
                equals,
            } => validate_values(expected, equals),

            Self::InclusiveRange {
                ref start,
                ref end,
                compare,
            } => match compare(start, end) {
                Some(Ordering::Less | Ordering::Equal) => Ok(()),

                Some(Ordering::Greater) | None => {
                    Err(MatcherValidationError::InvalidInclusiveRange)
                }
            },
        }
    }
}

/// Invalid declarative matcher definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatcherValidationError {
    /// A value-set bin contains no values.
    EmptyValueSet,

    /// A value-set bin repeats one value.
    DuplicateValue {
        /// Original value position.
        first: usize,

        /// Repeated value position.
        duplicate: usize,
    },

    /// Inclusive range bounds are reversed or incomparable.
    InvalidInclusiveRange,
}

/// Compares two values through `PartialEq`.
fn values_equal<T>(left: &T, right: &T) -> bool
where
    T: PartialEq,
{
    left == right
}

/// Compares two values through `PartialOrd`.
fn compare_values<T>(left: &T, right: &T) -> Option<Ordering>
where
    T: PartialOrd,
{
    left.partial_cmp(right)
}

/// Validates one value-set matcher.
fn validate_values<T>(
    values: &[T],
    equals: EqualityFunction<T>,
) -> Result<(), MatcherValidationError> {
    if values.is_empty() {
        return Err(MatcherValidationError::EmptyValueSet);
    }

    for (duplicate, value) in values.iter().enumerate() {
        let first = values
            .iter()
            .take(duplicate)
            .position(|candidate| equals(candidate, value));

        if let Some(first) = first {
            return Err(MatcherValidationError::DuplicateValue { first, duplicate });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::BinMatcher;
    use crate::Bits;
    use crate::coverage::MatcherValidationError;

    #[test]
    fn exact_value_matches() {
        let matcher = BinMatcher::value(7_u8);

        assert!(matcher.matches(&7));
    }

    #[test]
    fn exact_value_rejects_other_value() {
        let matcher = BinMatcher::value(7_u8);

        assert!(!matcher.matches(&8));
    }

    #[test]
    fn exact_value_supports_wide_bits() {
        let expected = Bits::<65>::from_words_le([0x0123_4567, 0x89ab_cdef, 1])
            .expect("correct wide bit-vector word count");
        let matcher = BinMatcher::value(expected.clone());

        assert!(matcher.matches(&expected));
        assert!(!matcher.matches(&Bits::<65>::zero()));
    }

    #[test]
    fn value_set_matches_each_member() {
        let matcher = BinMatcher::values([1_u8, 3, 5]);

        assert!(matcher.matches(&1));
        assert!(matcher.matches(&3));
        assert!(matcher.matches(&5));
    }

    #[test]
    fn value_set_rejects_non_member() {
        let matcher = BinMatcher::values([1_u8, 3, 5]);

        assert!(!matcher.matches(&2));
    }

    #[test]
    fn value_set_validation_rejects_empty_definition() {
        let matcher = BinMatcher::values(Vec::<u8>::new());

        assert_eq!(
            matcher.validate(),
            Err(MatcherValidationError::EmptyValueSet)
        );
    }

    #[test]
    fn value_set_validation_reports_duplicate_indices() {
        let matcher = BinMatcher::values(["Read", "Write", "Read"]);

        assert_eq!(
            matcher.validate(),
            Err(MatcherValidationError::DuplicateValue {
                first: 0,
                duplicate: 2,
            }),
        );
    }

    #[test]
    fn inclusive_range_matches_lower_bound() {
        assert!(BinMatcher::inclusive_range(2_u8, 4).matches(&2));
    }

    #[test]
    fn inclusive_range_matches_upper_bound() {
        assert!(BinMatcher::inclusive_range(2_u8, 4).matches(&4));
    }

    #[test]
    fn inclusive_range_matches_interior() {
        assert!(BinMatcher::inclusive_range(2_u8, 4).matches(&3));
    }

    #[test]
    fn inclusive_range_rejects_below() {
        assert!(!BinMatcher::inclusive_range(2_u8, 4).matches(&1));
    }

    #[test]
    fn inclusive_range_rejects_above() {
        assert!(!BinMatcher::inclusive_range(2_u8, 4).matches(&5));
    }

    #[test]
    fn inclusive_range_validation_rejects_reversed_bounds() {
        let matcher = BinMatcher::inclusive_range(4_u8, 2);

        assert_eq!(
            matcher.validate(),
            Err(MatcherValidationError::InvalidInclusiveRange)
        );
    }

    #[test]
    fn inclusive_range_validation_rejects_incomparable_bounds() {
        let matcher = BinMatcher::inclusive_range(f32::NAN, 1.0);

        assert_eq!(
            matcher.validate(),
            Err(MatcherValidationError::InvalidInclusiveRange)
        );
    }

    #[test]
    fn incomparable_sample_does_not_match() {
        let matcher = BinMatcher::inclusive_range(0.0_f32, 1.0);

        assert!(!matcher.matches(&f32::NAN));
    }
}
