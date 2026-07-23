use core::fmt;

use super::CrossBinId;
use crate::BinId;

/// Invalid two-way cross definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossBuildError {
    /// Cross name is invalid.
    InvalidName {
        /// Invalid name.
        name: String,
    },
    /// Generated cross bins have a zero hit threshold.
    ZeroRequiredHits {
        /// Cross name.
        cross: String,
    },
    /// Cross-bin cardinality limit is zero.
    ZeroBinLimit {
        /// Cross name.
        cross: String,
    },
    /// Axis cardinality multiplication overflowed.
    CardinalityOverflow {
        /// Cross name.
        cross: String,
        /// Left normal-bin count.
        left_bins: usize,
        /// Right normal-bin count.
        right_bins: usize,
    },
    /// Generated cardinality exceeds the configured bound.
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

impl fmt::Display for CrossBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidName { ref name } => {
                write!(f, "invalid functional coverage cross name `{name}`")
            }
            Self::ZeroRequiredHits { ref cross } => {
                write!(f, "cross `{cross}` requires a non-zero hit threshold")
            }
            Self::ZeroBinLimit { ref cross } => {
                write!(f, "cross `{cross}` requires a non-zero bin limit")
            }
            Self::CardinalityOverflow {
                ref cross,
                left_bins,
                right_bins,
            } => write!(
                f,
                "cross `{cross}` cardinality overflow: {left_bins} left bins × {right_bins} right \
                 bins"
            ),
            Self::BinLimitExceeded {
                ref cross,
                total_bins,
                limit,
                ..
            } => write!(
                f,
                "cross `{cross}` would generate {total_bins} bins, exceeding limit {limit}"
            ),
            Self::TooManyBins {
                ref cross,
                total_bins,
            } => {
                write!(f, "cross `{cross}` contains too many bins: {total_bins}")
            }
        }
    }
}

impl std::error::Error for CrossBuildError {}

/// Side of a two-way cross.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
