use core::fmt;

use crate::{BinId, CrossBinId};

/// Invalid two-way cross definition.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CrossBuildError {
    /// Cross name is invalid.
    #[error("invalid functional coverage cross name `{name}`")]
    InvalidName {
        /// Invalid name.
        name: String,
    },
    /// Generated cross bins have a zero hit threshold.
    #[error("cross `{cross}` requires a non-zero hit threshold")]
    ZeroRequiredHits {
        /// Cross name.
        cross: String,
    },
    /// Cross-bin cardinality limit is zero.
    #[error("cross `{cross}` requires a non-zero bin limit")]
    ZeroBinLimit {
        /// Cross name.
        cross: String,
    },
    /// Axis cardinality multiplication overflowed.
    #[error(
        "cross `{cross}` cardinality overflow: {left_bins} left bins × {right_bins} right bins"
    )]
    CardinalityOverflow {
        /// Cross name.
        cross: String,
        /// Left normal-bin count.
        left_bins: usize,
        /// Right normal-bin count.
        right_bins: usize,
    },
    /// Generated cardinality exceeds the configured bound.
    #[error("cross `{cross}` would generate {total_bins} bins, exceeding limit {limit}")]
    BinLimitExceeded {
        /// Cross name.
        cross: String,
        /// Left normal-bin count.
        left_bins: usize,
        /// Right normal-bin count.
        right_bins: usize,
        /// Requested generated-bin count.
        total_bins: usize,
        /// Configured maximum.
        limit: usize,
    },
    /// Generated bins cannot be represented by `CrossBinId`.
    #[error("cross `{cross}` contains too many bins: {total_bins}")]
    TooManyBins {
        /// Cross name.
        cross: String,
        /// Generated-bin count.
        total_bins: usize,
    },
}

impl CrossBuildError {
    /// Returns the affected cross name.
    #[must_use]
    pub fn cross(&self) -> Option<&str> {
        match *self {
            Self::InvalidName { .. } => None,
            Self::ZeroRequiredHits { ref cross }
            | Self::ZeroBinLimit { ref cross }
            | Self::CardinalityOverflow { ref cross, .. }
            | Self::BinLimitExceeded { ref cross, .. }
            | Self::TooManyBins { ref cross, .. } => Some(cross),
        }
    }
}

/// Side of a two-way cross.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CrossAxis {
    /// Left coverpoint.
    Left,
    /// Right coverpoint.
    Right,
}

impl fmt::Display for CrossAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Left => f.write_str("left"),
            Self::Right => f.write_str("right"),
        }
    }
}

/// Kind of two-way cross counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CrossCounterKind {
    /// Total cross sample calls.
    Samples,
    /// Samples skipped because one axis did not hit.
    SkippedSamples,
    /// One generated cross-bin hit count.
    BinHits,
}

impl fmt::Display for CrossCounterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Samples => f.write_str("total sample count"),
            Self::SkippedSamples => f.write_str("skipped sample count"),
            Self::BinHits => f.write_str("bin hit count"),
        }
    }
}

/// Failure while sampling a two-way cross.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CrossSampleError {
    /// Sample originated from a different coverpoint instance.
    SourceMismatch {
        /// Cross name.
        cross: String,
        /// Affected axis.
        axis: CrossAxis,
        /// Expected coverpoint name.
        expected: String,
        /// Actual coverpoint name.
        actual: String,
    },
    /// A source sample contains an unknown bin ID.
    UnknownBin {
        /// Cross name.
        cross: String,
        /// Affected axis.
        axis: CrossAxis,
        /// Source coverpoint name.
        coverpoint: String,
        /// Unknown source-bin ID.
        bin: BinId,
    },
    /// A cross counter could not be incremented.
    CounterOverflow {
        /// Cross name.
        cross: String,
        /// Cross bin for bin-local overflow.
        bin: Option<CrossBinId>,
        /// Counter kind.
        counter: CrossCounterKind,
    },
}

impl CrossSampleError {
    /// Returns the affected cross name.
    #[must_use]
    pub fn cross(&self) -> &str {
        match *self {
            Self::SourceMismatch { ref cross, .. }
            | Self::UnknownBin { ref cross, .. }
            | Self::CounterOverflow { ref cross, .. } => cross,
        }
    }
    /// Returns the affected axis when applicable.
    #[must_use]
    pub const fn axis(&self) -> Option<CrossAxis> {
        match *self {
            Self::SourceMismatch { axis, .. } | Self::UnknownBin { axis, .. } => Some(axis),
            Self::CounterOverflow { .. } => None,
        }
    }
    /// Returns the expected source name for a mismatch.
    #[must_use]
    pub fn expected(&self) -> Option<&str> {
        match *self {
            Self::SourceMismatch { ref expected, .. } => Some(expected),
            _ => None,
        }
    }
    /// Returns the actual source name for a mismatch.
    #[must_use]
    pub fn actual(&self) -> Option<&str> {
        match *self {
            Self::SourceMismatch { ref actual, .. } => Some(actual),
            _ => None,
        }
    }
    /// Returns the unknown source-bin ID when applicable.
    #[must_use]
    pub const fn source_bin(&self) -> Option<BinId> {
        match *self {
            Self::UnknownBin { bin, .. } => Some(bin),
            _ => None,
        }
    }
    /// Returns the overflowing cross-bin ID when applicable.
    #[must_use]
    pub const fn bin(&self) -> Option<CrossBinId> {
        match *self {
            Self::CounterOverflow { bin, .. } => bin,
            _ => None,
        }
    }
    /// Returns the overflowing counter kind when applicable.
    #[must_use]
    pub const fn counter(&self) -> Option<CrossCounterKind> {
        match *self {
            Self::CounterOverflow { counter, .. } => Some(counter),
            _ => None,
        }
    }
}

// This remains manual because optional bin context must appear in a fixed order.
// Counter overflow wording depends on whether a generated cross-bin ID exists.
impl fmt::Display for CrossSampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SourceMismatch {
                ref cross,
                axis,
                ref expected,
                ref actual,
            } => write!(
                f,
                "cross `{cross}` expected {axis} sample from `{expected}`, but received a sample \
                 from `{actual}`"
            ),
            Self::UnknownBin {
                ref cross,
                axis,
                ref coverpoint,
                ..
            } => write!(
                f,
                "cross `{cross}` received unknown {axis} bin ID from `{coverpoint}`"
            ),
            Self::CounterOverflow {
                ref cross,
                bin: Some(bin),
                counter,
            } => write!(
                f,
                "functional coverage cross counter overflow in `{cross}`: cross bin hit count \
                 ({bin:?}): {counter}"
            ),
            Self::CounterOverflow {
                ref cross,
                bin: None,
                counter,
            } => write!(
                f,
                "functional coverage cross counter overflow in `{cross}`: {counter}"
            ),
        }
    }
}

impl std::error::Error for CrossSampleError {}

#[cfg(test)]
mod tests {
    use super::CrossBuildError;
    use crate::{BinId, CrossAxis, CrossBinId, CrossCounterKind, CrossSampleError};

    #[test]
    fn build_errors_expose_the_affected_cross_when_available() {
        let invalid = CrossBuildError::InvalidName {
            name: "bad name".into(),
        };
        let limited = CrossBuildError::BinLimitExceeded {
            cross: "opcode_x_response".into(),
            left_bins: 2,
            right_bins: 3,
            total_bins: 6,
            limit: 4,
        };

        assert_eq!(invalid.cross(), None);
        assert_eq!(
            invalid.to_string(),
            "invalid functional coverage cross name `bad name`"
        );
        assert_eq!(limited.cross(), Some("opcode_x_response"));
        assert_eq!(
            limited.to_string(),
            "cross `opcode_x_response` would generate 6 bins, exceeding limit 4"
        );
    }

    #[test]
    fn sample_errors_return_variant_specific_context() {
        let unknown = CrossSampleError::UnknownBin {
            cross: "opcode_x_response".into(),
            axis: CrossAxis::Right,
            coverpoint: "response".into(),
            bin: BinId::new(7),
        };
        let overflow = CrossSampleError::CounterOverflow {
            cross: "opcode_x_response".into(),
            bin: Some(CrossBinId::new(2)),
            counter: CrossCounterKind::BinHits,
        };

        assert_eq!(unknown.cross(), "opcode_x_response");
        assert_eq!(unknown.axis(), Some(CrossAxis::Right));
        assert_eq!(unknown.source_bin(), Some(BinId::new(7)));
        assert_eq!(unknown.expected(), None);
        assert_eq!(unknown.actual(), None);
        assert_eq!(
            unknown.to_string(),
            "cross `opcode_x_response` received unknown right bin ID from `response`"
        );
        assert_eq!(overflow.axis(), None);
        assert_eq!(overflow.bin(), Some(CrossBinId::new(2)));
        assert_eq!(overflow.counter(), Some(CrossCounterKind::BinHits));
        assert_eq!(
            overflow.to_string(),
            "functional coverage cross counter overflow in `opcode_x_response`: cross bin hit \
             count (CrossBinId(2)): bin hit count"
        );
    }
}
