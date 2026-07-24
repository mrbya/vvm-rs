use std::fmt;
use std::path::PathBuf;

use crate::{CoverageDefinitionFingerprint, CoverageMergePolicy, CoveragePersistenceError};

/// Runtime counter merged across compatible artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug)]
pub enum CoverageMergeError {
    /// No per-test artifact was supplied.
    NoInputArtifacts,
    /// The policy excluded every artifact.
    NoIncludedArtifacts {
        /// Applied policy.
        policy: CoverageMergePolicy,
    },
    /// One exact file path appeared more than once.
    DuplicateInputPath {
        /// Duplicate path.
        path: PathBuf,
    },
    /// One artifact file could not be read or validated.
    ReadArtifact {
        /// Input file path.
        path: PathBuf,
        /// Nested persistence error.
        source: CoveragePersistenceError,
    },
    /// One included group path used incompatible definitions.
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
    CountOverflow {
        /// Overflowed summary count.
        counter: CoverageMergeCountKind,
    },
    /// Canonical ordering or artifact conversion failed.
    Persistence {
        /// Nested persistence error.
        source: CoveragePersistenceError,
    },
}

impl fmt::Display for CoverageMergeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NoInputArtifacts => formatter.write_str("no coverage artifacts were supplied"),
            Self::NoIncludedArtifacts { policy } => write!(
                formatter,
                "coverage merge policy `{policy}` excluded every artifact"
            ),
            Self::DuplicateInputPath { ref path } => write!(
                formatter,
                "duplicate coverage input path `{}`",
                path.display()
            ),
            Self::ReadArtifact {
                ref path,
                ref source,
            } => write!(
                formatter,
                "could not read coverage artifact `{}`: {source}",
                path.display()
            ),
            Self::IncompatibleDefinition {
                ref instance_path, ..
            } => write!(
                formatter,
                "incompatible coverage definitions for instance `{instance_path}`"
            ),
            Self::DefinitionStructureMismatch {
                ref instance_path,
                ref path,
                ref reason,
                ..
            } => write!(
                formatter,
                "coverage definition structure mismatch for instance `{instance_path}` at \
                 `{path}`: {reason}"
            ),
            Self::CounterOverflow {
                ref instance_path,
                ref item,
                counter,
                ..
            } => write!(
                formatter,
                "coverage merge overflow for {counter} in `{instance_path}.{item}`"
            ),
            Self::CountOverflow { counter } => {
                write!(formatter, "coverage merge summary overflow for {counter}")
            }
            Self::Persistence { ref source } => {
                write!(formatter, "coverage merge persistence failure: {source}")
            }
        }
    }
}

impl std::error::Error for CoverageMergeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ReadArtifact { ref source, .. } | Self::Persistence { ref source } => {
                Some(source)
            }
            _ => None,
        }
    }
}
