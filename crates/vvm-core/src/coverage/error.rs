use core::fmt;

use crate::coverage::{BinKind, CoverageCounterKind};

/// Invalid functional coverage definition errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageBuildError {
    /// Coverpoint name is invalid.
    InvalidCoverpointName {
        /// Provided name.
        name: String,
    },

    /// Bin name is invalid.
    InvalidBinName {
        /// Owning coverpoint.
        coverpoint: String,

        /// Provided bin name.
        bin: String,
    },

    /// Two bins share one name.
    DuplicateBinName {
        /// Owning coverpoint.
        coverpoint: String,

        /// Provided bin name.
        bin: String,
    },

    /// No normal bins were provided.
    NoNormalBins {
        /// Coverpoint name.
        coverpoint: String,
    },

    /// Value-set matcher contains no values.
    EmptyValueSet {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Value-set matcher repeats a value.
    DuplicateValue {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,

        /// First position.
        first: usize,

        /// Duplicate value position.
        duplicate: usize,
    },

    /// Inclusive range bounds are invalid.
    InvalidInclusiveRange {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Normal-bin hit threshold is zero.
    ZeroRequiredHits {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Ignore or illegal bin has a custom threshold.
    HitRequirementOnExcludedBin {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,

        /// Excluded bin kind.
        kind: BinKind,
    },

    /// More bins exist than `BinId` can represent.
    TooManyBins {
        /// Coverpoint name.
        coverpoint: String,
    },
}

impl fmt::Display for CoverageBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidCoverpointName { ref name } => {
                write!(f, "coverpoint name `{name}` is invalid")
            }
            Self::InvalidBinName {
                ref coverpoint,
                ref bin,
            } => write!(f, "coverpoint `{coverpoint}`, bin `{bin}` name is invalid"),
            Self::DuplicateBinName {
                ref coverpoint,
                ref bin,
            } => write!(f, "coverpoint `{coverpoint}` already contains bin `{bin}`"),
            Self::NoNormalBins { ref coverpoint } => {
                write!(f, "coverpoint `{coverpoint}` contains no normal bins")
            }
            Self::EmptyValueSet {
                ref coverpoint,
                ref bin,
            } => write!(
                f,
                "coverpoint `{coverpoint}`, bin `{bin}` contains no values to match against"
            ),
            Self::DuplicateValue {
                ref coverpoint,
                ref bin,
                ..
            } => write!(
                f,
                "coverpoint `{coverpoint}`, bin `{bin}` contains duplicate values"
            ),
            Self::InvalidInclusiveRange {
                ref coverpoint,
                ref bin,
            } => write!(
                f,
                "coverpoint `{coverpoint}`, bin `{bin}` inclusive range boundaries are invalid"
            ),
            Self::ZeroRequiredHits {
                ref coverpoint,
                ref bin,
            } => write!(
                f,
                "coverpoint `{coverpoint}`, normal bin `{bin}` hit threshold is zero"
            ),
            Self::HitRequirementOnExcludedBin {
                ref coverpoint,
                ref bin,
                kind,
            } => write!(
                f,
                "coverpoint `{coverpoint}`, excluded {kind} bin `{bin}` has a non-zero hit \
                 requirement"
            ),
            Self::TooManyBins { ref coverpoint } => {
                write!(f, "coverpoint `{coverpoint}` contains too many bins")
            }
        }
    }
}

impl std::error::Error for CoverageBuildError {}

/// Failure while sampling functional coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageSampleError {
    /// One or more illegal bins matched.
    IllegalBinHit {
        /// Coverpoint name.
        coverpoint: String,

        /// Matching illegal bins.
        bins: Box<[String]>,

        /// Debug representation of the sample.
        sampled_value: String,
    },

    /// One counter could not be incremented.
    CounterOverflow {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name for bin-local counters.
        bin: Option<String>,

        /// Counter that overflowed.
        counter: CoverageCounterKind,
    },
}

impl CoverageSampleError {
    /// Returns the affected coverpoint name.
    #[must_use]
    pub fn coverpoint(&self) -> &str {
        match *self {
            Self::IllegalBinHit { ref coverpoint, .. }
            | Self::CounterOverflow { ref coverpoint, .. } => coverpoint,
        }
    }

    /// Returns the overflowing bin name, when applicable.
    #[must_use]
    pub fn bin(&self) -> Option<&str> {
        match *self {
            Self::CounterOverflow { ref bin, .. } => bin.as_deref(),
            Self::IllegalBinHit { .. } => None,
        }
    }

    /// Returns the overflowing counter kind.
    #[must_use]
    pub const fn counter(&self) -> Option<CoverageCounterKind> {
        match *self {
            Self::CounterOverflow { ref counter, .. } => Some(*counter),
            Self::IllegalBinHit { .. } => None,
        }
    }

    /// Returns matching illegal-bin names.
    #[must_use]
    pub fn illegal_bins(&self) -> Option<&[String]> {
        match *self {
            Self::IllegalBinHit { ref bins, .. } => Some(bins),
            Self::CounterOverflow { .. } => None,
        }
    }

    /// Returns the formatted illegal sampled value.
    #[must_use]
    pub fn sampled_value(&self) -> Option<&str> {
        match *self {
            Self::IllegalBinHit {
                ref sampled_value, ..
            } => Some(sampled_value),
            Self::CounterOverflow { .. } => None,
        }
    }
}

impl std::fmt::Display for CoverageSampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::IllegalBinHit {
                ref coverpoint,
                ref bins,
                ref sampled_value,
            } => {
                write!(f, "illegal coverage bin hit in `{coverpoint}`: bins [")?;

                for (index, bin) in bins.iter().enumerate() {
                    if index != 0 {
                        f.write_str(", ")?;
                    }

                    write!(f, "`{bin}`")?;
                }

                write!(f, "], sampled value {sampled_value}")
            }

            Self::CounterOverflow {
                ref coverpoint,
                ref bin,
                ref counter,
            } => {
                if let Some(bin) = bin.as_deref() {
                    write!(
                        f,
                        "functional coverage counter overflow in `{coverpoint}.{bin}`: {counter}",
                    )
                } else {
                    write!(
                        f,
                        "functional coverage counter overflow in `{coverpoint}`: {counter}",
                    )
                }
            }
        }
    }
}

impl std::error::Error for CoverageSampleError {}
