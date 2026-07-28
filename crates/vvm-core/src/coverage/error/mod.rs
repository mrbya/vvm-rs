/// Coverage artifact persistence errors.
mod artifact;
/// Cross construction and sampling errors.
mod cross;
/// Coverage-group construction errors.
mod group;
/// Coverage merge errors.
mod merge;
/// Typed coverage model construction and sampling errors.
mod model;
/// Coverage session errors.
mod session;

pub use artifact::{CoverageIoOperation, CoveragePersistenceError};
pub use cross::{CrossAxis, CrossBuildError, CrossCounterKind, CrossSampleError};
pub use group::{CoverageGroupCountKind, CoverageGroupError};
pub use merge::{CoverageMergeCountKind, CoverageMergeCounterKind, CoverageMergeError};
pub use model::{CoverageDefinitionError, CoverageRuntimeError, CoverageRuntimeItemKind};
pub use session::{CoverageSessionCountKind, CoverageSessionError};

use crate::coverage::{BinKind, CoverageCounterKind};

/// Invalid functional coverage definition errors.
///
/// These errors report invalid definitions during [`crate::CoverpointBuilder::build`],
/// before runtime sampling begins.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoverageBuildError {
    /// Coverpoint name is invalid.
    #[error("coverpoint name `{name}` is invalid")]
    InvalidCoverpointName {
        /// Provided name.
        name: String,
    },

    /// Bin name is invalid.
    #[error("coverpoint `{coverpoint}`, bin `{bin}` name is invalid")]
    InvalidBinName {
        /// Owning coverpoint.
        coverpoint: String,

        /// Provided bin name.
        bin: String,
    },

    /// Two bins share one name.
    #[error("coverpoint `{coverpoint}` already contains bin `{bin}`")]
    DuplicateBinName {
        /// Owning coverpoint.
        coverpoint: String,

        /// Provided bin name.
        bin: String,
    },

    /// No normal bins were provided.
    #[error("coverpoint `{coverpoint}` contains no normal bins")]
    NoNormalBins {
        /// Coverpoint name.
        coverpoint: String,
    },

    /// Value-set matcher contains no values.
    #[error("coverpoint `{coverpoint}`, bin `{bin}` contains no values to match against")]
    EmptyValueSet {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Value-set matcher repeats a value.
    #[error("coverpoint `{coverpoint}`, bin `{bin}` contains duplicate values")]
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
    #[error("coverpoint `{coverpoint}`, bin `{bin}` inclusive range boundaries are invalid")]
    InvalidInclusiveRange {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Normal-bin hit threshold is zero.
    #[error("coverpoint `{coverpoint}`, normal bin `{bin}` hit threshold is zero")]
    ZeroRequiredHits {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,
    },

    /// Ignore or illegal bin has a custom threshold.
    #[error(
        "coverpoint `{coverpoint}`, excluded {kind} bin `{bin}` has a non-zero hit requirement"
    )]
    HitRequirementOnExcludedBin {
        /// Coverpoint name.
        coverpoint: String,

        /// Bin name.
        bin: String,

        /// Excluded bin kind.
        kind: BinKind,
    },

    /// More bins exist than `BinId` can represent.
    #[error("coverpoint `{coverpoint}` contains too many bins")]
    TooManyBins {
        /// Coverpoint name.
        coverpoint: String,
    },
}

/// Failure while sampling functional coverage.
///
/// [`CoverageSampleError::IllegalBinHit`] is a semantic invalid-coverage event:
/// its counters are recorded before the error is returned. In contrast,
/// [`CoverageSampleError::CounterOverflow`] means an update cannot be
/// represented and no counter mutation occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

// This remains manual because illegal bins must be formatted in their sample order.
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

#[cfg(test)]
mod tests {
    use crate::{BinKind, CoverageBuildError, CoverageCounterKind, CoverageSampleError};

    #[test]
    fn formats_invalid_coverpoint_name() {
        assert_eq!(
            CoverageBuildError::InvalidCoverpointName {
                name: "bad name".into()
            }
            .to_string(),
            "coverpoint name `bad name` is invalid"
        );
    }

    #[test]
    fn formats_invalid_bin_name() {
        assert_eq!(
            CoverageBuildError::InvalidBinName {
                coverpoint: "opcode".into(),
                bin: "bad name".into()
            }
            .to_string(),
            "coverpoint `opcode`, bin `bad name` name is invalid"
        );
    }

    #[test]
    fn formats_duplicate_bin_name() {
        assert_eq!(
            CoverageBuildError::DuplicateBinName {
                coverpoint: "opcode".into(),
                bin: "read".into()
            }
            .to_string(),
            "coverpoint `opcode` already contains bin `read`"
        );
    }

    #[test]
    fn formats_missing_normal_bin() {
        assert_eq!(
            CoverageBuildError::NoNormalBins {
                coverpoint: "opcode".into()
            }
            .to_string(),
            "coverpoint `opcode` contains no normal bins"
        );
    }

    #[test]
    fn formats_empty_value_set() {
        assert_eq!(
            CoverageBuildError::EmptyValueSet {
                coverpoint: "opcode".into(),
                bin: "reads".into()
            }
            .to_string(),
            "coverpoint `opcode`, bin `reads` contains no values to match against"
        );
    }

    #[test]
    fn formats_duplicate_value_indices() {
        let error = CoverageBuildError::DuplicateValue {
            coverpoint: "opcode".into(),
            bin: "reads".into(),
            first: 0,
            duplicate: 2,
        };

        assert_eq!(
            error.to_string(),
            "coverpoint `opcode`, bin `reads` contains duplicate values"
        );
    }

    #[test]
    fn formats_invalid_range() {
        assert_eq!(
            CoverageBuildError::InvalidInclusiveRange {
                coverpoint: "opcode".into(),
                bin: "range".into()
            }
            .to_string(),
            "coverpoint `opcode`, bin `range` inclusive range boundaries are invalid"
        );
    }

    #[test]
    fn formats_zero_hit_threshold() {
        assert_eq!(
            CoverageBuildError::ZeroRequiredHits {
                coverpoint: "opcode".into(),
                bin: "read".into()
            }
            .to_string(),
            "coverpoint `opcode`, normal bin `read` hit threshold is zero"
        );
    }

    #[test]
    fn formats_excluded_bin_hit_threshold() {
        let error = CoverageBuildError::HitRequirementOnExcludedBin {
            coverpoint: "opcode".into(),
            bin: "reset".into(),
            kind: BinKind::Ignore,
        };

        assert_eq!(
            error.to_string(),
            "coverpoint `opcode`, excluded ignore bin `reset` has a non-zero hit requirement"
        );
    }

    #[test]
    fn formats_too_many_bins() {
        assert_eq!(
            CoverageBuildError::TooManyBins {
                coverpoint: "opcode".into()
            }
            .to_string(),
            "coverpoint `opcode` contains too many bins"
        );
    }

    #[test]
    fn formats_single_illegal_bin_hit() {
        assert_eq!(
            illegal_error(["reserved"]).to_string(),
            "illegal coverage bin hit in `opcode`: bins [`reserved`], sampled value 255"
        );
    }

    #[test]
    fn formats_multiple_illegal_bin_hits() {
        assert_eq!(
            illegal_error(["reserved_a", "reserved_b"]).to_string(),
            "illegal coverage bin hit in `opcode`: bins [`reserved_a`, `reserved_b`], sampled \
             value 255"
        );
    }

    #[test]
    fn formats_coverpoint_counter_overflow() {
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).to_string(),
            "functional coverage counter overflow in `opcode`: total sample count"
        );
    }

    #[test]
    fn formats_bin_counter_overflow() {
        assert_eq!(
            overflow_error(Some("read"), CoverageCounterKind::BinHits).to_string(),
            "functional coverage counter overflow in `opcode.read`: bin hit count"
        );
    }

    #[test]
    fn sample_error_returns_coverpoint_name() {
        assert_eq!(illegal_error(["reserved"]).coverpoint(), "opcode");
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).coverpoint(),
            "opcode"
        );
    }

    #[test]
    fn illegal_error_returns_illegal_bin_names() {
        assert_eq!(
            illegal_error(["reserved"]).illegal_bins(),
            Some(["reserved".into()].as_slice())
        );
    }

    #[test]
    fn illegal_error_returns_sampled_value() {
        assert_eq!(illegal_error(["reserved"]).sampled_value(), Some("255"));
    }

    #[test]
    fn illegal_error_has_no_counter_kind() {
        assert_eq!(illegal_error(["reserved"]).counter(), None);
    }

    #[test]
    fn illegal_error_has_no_overflow_bin() {
        assert_eq!(illegal_error(["reserved"]).bin(), None);
    }

    #[test]
    fn overflow_error_returns_counter_kind() {
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).counter(),
            Some(CoverageCounterKind::Samples)
        );
    }

    #[test]
    fn bin_overflow_returns_bin_name() {
        assert_eq!(
            overflow_error(Some("read"), CoverageCounterKind::BinHits).bin(),
            Some("read")
        );
    }

    #[test]
    fn coverpoint_overflow_has_no_bin_name() {
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).bin(),
            None
        );
    }

    #[test]
    fn overflow_error_has_no_illegal_bins() {
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).illegal_bins(),
            None
        );
    }

    #[test]
    fn overflow_error_has_no_sampled_value() {
        assert_eq!(
            overflow_error(None, CoverageCounterKind::Samples).sampled_value(),
            None
        );
    }

    fn illegal_error<const N: usize>(bins: [&str; N]) -> CoverageSampleError {
        CoverageSampleError::IllegalBinHit {
            coverpoint: "opcode".into(),
            bins: bins.into_iter().map(str::to_owned).collect(),
            sampled_value: "255".into(),
        }
    }

    fn overflow_error(bin: Option<&str>, counter: CoverageCounterKind) -> CoverageSampleError {
        CoverageSampleError::CounterOverflow {
            coverpoint: "opcode".into(),
            bin: bin.map(str::to_owned),
            counter,
        }
    }
}
