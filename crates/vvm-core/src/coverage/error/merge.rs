use std::fmt;
use std::path::PathBuf;

use crate::{CoverageDefinitionFingerprint, CoverageMergePolicy, CoveragePersistenceError};

/// Runtime counter merged across compatible artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CoverageMergeCounterKind {
    /// Coverpoint attempted samples.
    CoverpointSamples,
    /// Coverpoint ignored samples.
    IgnoredSamples,
    /// Coverpoint illegal samples.
    IllegalSamples,
    /// Coverpoint unmatched samples.
    UnmatchedSamples,
    /// Coverpoint bin hits.
    CoverpointBinHits,
    /// Cross attempted samples.
    CrossSamples,
    /// Cross skipped samples.
    SkippedCrossSamples,
    /// Cross bin hits.
    CrossBinHits,
}

impl fmt::Display for CoverageMergeCounterKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match *self {
            Self::CoverpointSamples => "coverpoint samples",
            Self::IgnoredSamples => "ignored samples",
            Self::IllegalSamples => "illegal samples",
            Self::UnmatchedSamples => "unmatched samples",
            Self::CoverpointBinHits => "coverpoint bin hits",
            Self::CrossSamples => "cross samples",
            Self::SkippedCrossSamples => "skipped cross samples",
            Self::CrossBinHits => "cross bin hits",
        })
    }
}

/// Checked structural or provenance count in a merged result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CoverageMergeCountKind {
    /// Input artifacts.
    Artifacts,
    /// Included artifacts.
    IncludedArtifacts,
    /// Excluded artifacts.
    ExcludedArtifacts,
    /// Passed artifacts.
    PassedArtifacts,
    /// Failed artifacts.
    FailedArtifacts,
    /// Errored artifacts.
    ErroredArtifacts,
    /// Merged groups.
    Groups,
    /// Merged items.
    Items,
    /// Merged coverpoints.
    Coverpoints,
    /// Merged crosses.
    Crosses,
    /// Covered bins.
    CoveredBins,
    /// Uncovered bins.
    UncoveredBins,
    /// Total bins.
    TotalBins,
}

impl fmt::Display for CoverageMergeCountKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match *self {
            Self::Artifacts => "artifacts",
            Self::IncludedArtifacts => "included artifacts",
            Self::ExcludedArtifacts => "excluded artifacts",
            Self::PassedArtifacts => "passed artifacts",
            Self::FailedArtifacts => "failed artifacts",
            Self::ErroredArtifacts => "errored artifacts",
            Self::Groups => "groups",
            Self::Items => "items",
            Self::Coverpoints => "coverpoints",
            Self::Crosses => "crosses",
            Self::CoveredBins => "covered bins",
            Self::UncoveredBins => "uncovered bins",
            Self::TotalBins => "total bins",
        })
    }
}

/// Failure while combining validated functional-coverage artifacts.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CoverageMergeError {
    /// No per-test artifact was supplied.
    #[error("no coverage artifacts were supplied")]
    NoInputArtifacts,
    /// The policy excluded every artifact.
    #[error("coverage merge policy `{policy}` excluded every artifact")]
    NoIncludedArtifacts {
        /// Applied policy.
        policy: CoverageMergePolicy,
    },
    /// One exact file path appeared more than once.
    #[error("duplicate coverage input path `{}`", path.display())]
    DuplicateInputPath {
        /// Duplicate path.
        path: PathBuf,
    },
    /// One artifact file could not be read or validated.
    #[error("could not read coverage artifact `{}`: {source}", path.display())]
    ReadArtifact {
        /// Input file path.
        path: PathBuf,
        /// Nested persistence error.
        source: CoveragePersistenceError,
    },
    /// One included group path used incompatible definitions.
    #[error("incompatible coverage definitions for instance `{instance_path}`")]
    IncompatibleDefinition {
        /// Conflicting group instance path.
        instance_path: Box<str>,
        /// Canonically earlier test.
        existing_test: Box<str>,
        /// Incoming test.
        incoming_test: Box<str>,
        /// Existing definition name.
        existing_definition: Box<str>,
        /// Incoming definition name.
        incoming_definition: Box<str>,
        /// Existing definition revision.
        existing_revision: u64,
        /// Incoming definition revision.
        incoming_revision: u64,
        /// Existing structural fingerprint.
        existing_fingerprint: Box<CoverageDefinitionFingerprint>,
        /// Incoming structural fingerprint.
        incoming_fingerprint: Box<CoverageDefinitionFingerprint>,
    },
    /// Matching fingerprints were paired with unequal structure.
    #[error(
        "coverage definition structure mismatch for instance `{instance_path}` at `{path}`: \
         {reason}"
    )]
    DefinitionStructureMismatch {
        /// Group instance path.
        instance_path: String,
        /// Shared fingerprint.
        fingerprint: CoverageDefinitionFingerprint,
        /// Logical structure path.
        path: String,
        /// Mismatch reason.
        reason: String,
    },
    /// A runtime counter could not be summed.
    #[error("coverage merge overflow for {counter} in `{instance_path}.{item}`")]
    CounterOverflow {
        /// Group instance path.
        instance_path: String,
        /// Item name.
        item: String,
        /// Optional bin name or pair description.
        bin: Option<String>,
        /// Overflowed counter.
        counter: CoverageMergeCounterKind,
    },
    /// A result-summary count could not be represented.
    #[error("coverage merge summary overflow for {counter}")]
    CountOverflow {
        /// Overflowed summary count.
        counter: CoverageMergeCountKind,
    },
    /// Canonical ordering or artifact conversion failed.
    #[error("coverage merge persistence failure: {source}")]
    Persistence {
        /// Nested persistence error.
        source: CoveragePersistenceError,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        CoverageMergeCountKind, CoverageMergeCounterKind, CoverageMergeError, CoverageMergePolicy,
    };

    #[test]
    fn merge_counter_names_are_stable() {
        let counters = [
            (
                CoverageMergeCounterKind::CoverpointSamples,
                "coverpoint samples",
            ),
            (CoverageMergeCounterKind::IgnoredSamples, "ignored samples"),
            (CoverageMergeCounterKind::IllegalSamples, "illegal samples"),
            (
                CoverageMergeCounterKind::UnmatchedSamples,
                "unmatched samples",
            ),
            (
                CoverageMergeCounterKind::CoverpointBinHits,
                "coverpoint bin hits",
            ),
            (CoverageMergeCounterKind::CrossSamples, "cross samples"),
            (
                CoverageMergeCounterKind::SkippedCrossSamples,
                "skipped cross samples",
            ),
            (CoverageMergeCounterKind::CrossBinHits, "cross bin hits"),
        ];

        for (counter, expected) in counters {
            assert_eq!(counter.to_string(), expected);
        }
    }

    #[test]
    fn merge_errors_format_policy_path_and_overflow_context() {
        let excluded = CoverageMergeError::NoIncludedArtifacts {
            policy: CoverageMergePolicy::PassedOnly,
        };
        let duplicate = CoverageMergeError::DuplicateInputPath {
            path: PathBuf::from("coverage/decoder.json"),
        };
        let overflow = CoverageMergeError::CounterOverflow {
            instance_path: "dut.decoder".into(),
            item: "opcode".into(),
            bin: Some("read".into()),
            counter: CoverageMergeCounterKind::CoverpointBinHits,
        };
        let summary = CoverageMergeError::CountOverflow {
            counter: CoverageMergeCountKind::IncludedArtifacts,
        };

        assert_eq!(
            excluded.to_string(),
            "coverage merge policy `passed_only` excluded every artifact"
        );
        assert_eq!(
            duplicate.to_string(),
            "duplicate coverage input path `coverage/decoder.json`"
        );
        assert_eq!(
            overflow.to_string(),
            "coverage merge overflow for coverpoint bin hits in `dut.decoder.opcode`"
        );
        assert_eq!(
            summary.to_string(),
            "coverage merge summary overflow for included artifacts"
        );
    }
}
