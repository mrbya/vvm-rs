use core::fmt;

use crate::CrossAxis;

/// Checked count accumulated while inspecting a coverage group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageGroupCountKind {
    /// Total exposed items.
    Items,
    /// Typed coverpoints.
    Coverpoints,
    /// Two-way crosses.
    Crosses,
    /// Covered bins.
    CoveredBins,
    /// Uncovered bins.
    UncoveredBins,
    /// Total bins.
    TotalBins,
}

impl fmt::Display for CoverageGroupCountKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Items => f.write_str("item count"),
            Self::Coverpoints => f.write_str("coverpoint count"),
            Self::Crosses => f.write_str("cross count"),
            Self::CoveredBins => f.write_str("covered-bin count"),
            Self::UncoveredBins => f.write_str("uncovered-bin count"),
            Self::TotalBins => f.write_str("total-bin count"),
        }
    }
}

/// Invalid coverage-group identity, structure, or metric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageGroupError {
    /// Definition name is invalid.
    InvalidDefinitionName {
        /// Invalid definition name.
        name: String,
    },
    /// Hierarchical instance path is invalid.
    InvalidInstancePath {
        /// Invalid path.
        path: String,
    },
    /// Group exposes no coverage items.
    EmptyGroup {
        /// Definition name.
        definition: String,
        /// Instance path.
        instance: String,
    },
    /// Two items share one name.
    DuplicateItemName {
        /// Definition name.
        definition: String,
        /// Instance path.
        instance: String,
        /// Duplicate item name.
        item: String,
    },
    /// One exact coverpoint instance is exposed more than once.
    DuplicateCoverpoint {
        /// Definition name.
        definition: String,
        /// Instance path.
        instance: String,
        /// Coverpoint name.
        coverpoint: String,
    },
    /// Cross source coverpoint is absent from the group.
    MissingCrossSource {
        /// Definition name.
        definition: String,
        /// Instance path.
        instance: String,
        /// Cross name.
        cross: String,
        /// Missing axis.
        axis: CrossAxis,
        /// Missing source coverpoint name.
        coverpoint: String,
    },
    /// An aggregate count could not be represented.
    CountOverflow {
        /// Definition name.
        definition: String,
        /// Instance path.
        instance: String,
        /// Counter kind.
        counter: CoverageGroupCountKind,
    },
}

impl CoverageGroupError {
    /// Returns the definition name when available.
    #[must_use]
    pub fn definition(&self) -> Option<&str> {
        match *self {
            Self::InvalidDefinitionName { .. } | Self::InvalidInstancePath { .. } => None,
            Self::EmptyGroup { ref definition, .. }
            | Self::DuplicateItemName { ref definition, .. }
            | Self::DuplicateCoverpoint { ref definition, .. }
            | Self::MissingCrossSource { ref definition, .. }
            | Self::CountOverflow { ref definition, .. } => Some(definition),
        }
    }
    /// Returns the instance path when available.
    #[must_use]
    pub fn instance(&self) -> Option<&str> {
        match *self {
            Self::InvalidDefinitionName { .. } => None,
            Self::InvalidInstancePath { ref path } => Some(path),
            Self::EmptyGroup { ref instance, .. }
            | Self::DuplicateItemName { ref instance, .. }
            | Self::DuplicateCoverpoint { ref instance, .. }
            | Self::MissingCrossSource { ref instance, .. }
            | Self::CountOverflow { ref instance, .. } => Some(instance),
        }
    }
    /// Returns the duplicate item name when applicable.
    #[must_use]
    pub fn item(&self) -> Option<&str> {
        if let Self::DuplicateItemName { ref item, .. } = *self {
            Some(item)
        } else {
            None
        }
    }
    /// Returns the affected cross name when applicable.
    #[must_use]
    pub fn cross(&self) -> Option<&str> {
        if let Self::MissingCrossSource { ref cross, .. } = *self {
            Some(cross)
        } else {
            None
        }
    }
    /// Returns the missing cross axis when applicable.
    #[must_use]
    pub const fn axis(&self) -> Option<CrossAxis> {
        if let Self::MissingCrossSource { axis, .. } = *self {
            Some(axis)
        } else {
            None
        }
    }
    /// Returns the affected coverpoint name when applicable.
    #[must_use]
    pub fn coverpoint(&self) -> Option<&str> {
        match *self {
            Self::DuplicateCoverpoint { ref coverpoint, .. }
            | Self::MissingCrossSource { ref coverpoint, .. } => Some(coverpoint),
            _ => None,
        }
    }
    /// Returns the overflowing count kind when applicable.
    #[must_use]
    pub const fn counter(&self) -> Option<CoverageGroupCountKind> {
        if let Self::CountOverflow { counter, .. } = *self {
            Some(counter)
        } else {
            None
        }
    }
}

impl fmt::Display for CoverageGroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidDefinitionName { ref name } => {
                write!(f, "invalid coverage-group definition name `{name}`")
            }
            Self::InvalidInstancePath { ref path } => {
                write!(f, "invalid coverage-group instance path `{path}`")
            }
            Self::EmptyGroup {
                ref definition,
                ref instance,
            } => write!(
                f,
                "coverage group `{definition}` instance `{instance}` contains no coverage items"
            ),
            Self::DuplicateItemName {
                ref definition,
                ref instance,
                ref item,
            } => write!(
                f,
                "coverage group `{definition}` instance `{instance}` contains duplicate item name \
                 `{item}`"
            ),
            Self::DuplicateCoverpoint {
                ref definition,
                ref instance,
                ref coverpoint,
            } => write!(
                f,
                "coverage group `{definition}` instance `{instance}` exposes coverpoint \
                 `{coverpoint}` more than once"
            ),
            Self::MissingCrossSource {
                ref definition,
                ref instance,
                ref cross,
                axis,
                ref coverpoint,
            } => write!(
                f,
                "cross `{cross}` in coverage group `{definition}` instance `{instance}` \
                 references missing {axis} coverpoint `{coverpoint}`"
            ),
            Self::CountOverflow {
                ref definition,
                ref instance,
                counter,
            } => write!(
                f,
                "coverage group `{definition}` instance `{instance}` overflowed {counter}"
            ),
        }
    }
}

impl std::error::Error for CoverageGroupError {}
