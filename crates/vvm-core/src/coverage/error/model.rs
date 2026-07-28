use crate::{
    CoverageBuildError, CoverageGroupError, CoverageSampleError, CrossBuildError, CrossSampleError,
    SimulationTime,
};

/// Failure while constructing one typed functional-coverage model.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoverageDefinitionError {
    /// One generated coverpoint field failed to build.
    #[error("failed to build coverage coverpoint `{item}`: {source}")]
    Coverpoint {
        /// Declared field and item name.
        item: &'static str,
        /// Underlying coverpoint build error.
        source: CoverageBuildError,
    },
    /// One generated cross field failed to build.
    #[error("failed to build coverage cross `{item}`: {source}")]
    Cross {
        /// Declared field and item name.
        item: &'static str,
        /// Underlying cross build error.
        source: CrossBuildError,
    },
    /// Group identity or completed-group validation failed.
    #[error("failed to construct coverage group: {source}")]
    Group {
        /// Underlying coverage-group error.
        source: CoverageGroupError,
    },
}

impl CoverageDefinitionError {
    /// Returns the declared coverage item when construction reached one.
    #[must_use]
    pub const fn item(&self) -> Option<&'static str> {
        match *self {
            Self::Coverpoint { item, .. } | Self::Cross { item, .. } => Some(item),
            Self::Group { .. } => None,
        }
    }

    /// Returns the underlying coverpoint build error when applicable.
    #[must_use]
    pub const fn coverpoint_source(&self) -> Option<&CoverageBuildError> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { .. } | Self::Group { .. } => None,
        }
    }

    /// Returns the underlying cross build error when applicable.
    #[must_use]
    pub const fn cross_source(&self) -> Option<&CrossBuildError> {
        match *self {
            Self::Cross { ref source, .. } => Some(source),
            Self::Coverpoint { .. } | Self::Group { .. } => None,
        }
    }

    /// Returns the underlying group error when applicable.
    #[must_use]
    pub const fn group_source(&self) -> Option<&CoverageGroupError> {
        match *self {
            Self::Group { ref source } => Some(source),
            Self::Coverpoint { .. } | Self::Cross { .. } => None,
        }
    }
}

/// Kind of coverage item involved in a runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CoverageRuntimeItemKind {
    /// Coverpoint sampling.
    Coverpoint,
    /// Cross sampling.
    Cross,
}

/// Failure while sampling one typed functional-coverage model.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoverageRuntimeError {
    /// One coverpoint failed for an observed transaction.
    #[error("coverage coverpoint `{item}` failed at cycle {cycle} at {time}: {source}")]
    Coverpoint {
        /// Declared coverage item name.
        item: &'static str,
        /// Zero-based transaction cycle.
        cycle: u64,
        /// Observation sampling time.
        time: SimulationTime,
        /// Underlying coverpoint error.
        source: CoverageSampleError,
    },
    /// One cross failed for an observed transaction.
    #[error("coverage cross `{item}` failed at cycle {cycle} at {time}: {source}")]
    Cross {
        /// Declared coverage item name.
        item: &'static str,
        /// Zero-based transaction cycle.
        cycle: u64,
        /// Observation sampling time.
        time: SimulationTime,
        /// Underlying cross error.
        source: CrossSampleError,
    },
}

impl CoverageRuntimeError {
    /// Returns the declared coverage item name.
    #[must_use]
    pub const fn item(&self) -> &'static str {
        match *self {
            Self::Coverpoint { item, .. } | Self::Cross { item, .. } => item,
        }
    }

    /// Returns the zero-based observed transaction cycle.
    #[must_use]
    pub const fn cycle(&self) -> u64 {
        match *self {
            Self::Coverpoint { cycle, .. } | Self::Cross { cycle, .. } => cycle,
        }
    }

    /// Returns the observation sampling time.
    #[must_use]
    pub const fn time(&self) -> SimulationTime {
        match *self {
            Self::Coverpoint { time, .. } | Self::Cross { time, .. } => time,
        }
    }

    /// Returns the coverage item kind.
    #[must_use]
    pub const fn item_kind(&self) -> CoverageRuntimeItemKind {
        match *self {
            Self::Coverpoint { .. } => CoverageRuntimeItemKind::Coverpoint,
            Self::Cross { .. } => CoverageRuntimeItemKind::Cross,
        }
    }

    /// Returns the underlying coverpoint sampling error when applicable.
    #[must_use]
    pub const fn coverpoint_source(&self) -> Option<&CoverageSampleError> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { .. } => None,
        }
    }

    /// Returns the underlying cross sampling error when applicable.
    #[must_use]
    pub const fn cross_source(&self) -> Option<&CrossSampleError> {
        match *self {
            Self::Cross { ref source, .. } => Some(source),
            Self::Coverpoint { .. } => None,
        }
    }
}
