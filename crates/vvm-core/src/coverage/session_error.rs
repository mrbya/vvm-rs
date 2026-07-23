use core::fmt;

use crate::CoverageGroupError;

/// Aggregate count maintained by a per-test coverage session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageSessionCountKind {
    /// Captured group instances.
    Groups,
    /// Captured coverage items.
    Items,
    /// Captured typed coverpoints.
    Coverpoints,
    /// Captured two-way crosses.
    Crosses,
    /// Covered normal bins.
    CoveredBins,
    /// Uncovered normal bins.
    UncoveredBins,
    /// Total normal bins.
    TotalBins,
}
impl fmt::Display for CoverageSessionCountKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Groups => "group count",
            Self::Items => "item count",
            Self::Coverpoints => "coverpoint count",
            Self::Crosses => "cross count",
            Self::CoveredBins => "covered-bin count",
            Self::UncoveredBins => "uncovered-bin count",
            Self::TotalBins => "total-bin count",
        })
    }
}

/// Failure while constructing or capturing a coverage session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageSessionError {
    /// Test name is invalid.
    InvalidTestName {
        /// Invalid test name.
        name: String,
    },
    /// Captured group is structurally invalid.
    InvalidGroup {
        /// Associated test name.
        test: String,
        /// Nested group validation failure.
        source: Box<CoverageGroupError>,
    },
    /// An instance path has already been captured.
    DuplicateInstancePath {
        /// Associated test name.
        test: String,
        /// Repeated hierarchical instance path.
        instance_path: String,
    },
    /// An aggregate session counter overflowed.
    CountOverflow {
        /// Associated test name.
        test: String,
        /// Overflowing aggregate counter.
        counter: CoverageSessionCountKind,
    },
}
impl CoverageSessionError {
    /// Returns the associated test name.
    #[must_use]
    pub fn test(&self) -> &str {
        match *self {
            Self::InvalidTestName { ref name } => name,
            Self::InvalidGroup { ref test, .. }
            | Self::DuplicateInstancePath { ref test, .. }
            | Self::CountOverflow { ref test, .. } => test,
        }
    }
    /// Returns the duplicate instance path when applicable.
    #[must_use]
    pub fn instance_path(&self) -> Option<&str> {
        if let Self::DuplicateInstancePath {
            ref instance_path, ..
        } = *self
        {
            Some(instance_path)
        } else {
            None
        }
    }
    /// Returns the nested group error when applicable.
    #[must_use]
    pub const fn group_error(&self) -> Option<&CoverageGroupError> {
        if let Self::InvalidGroup { ref source, .. } = *self {
            Some(source)
        } else {
            None
        }
    }
    /// Returns the overflowing count kind when applicable.
    #[must_use]
    pub const fn counter(&self) -> Option<CoverageSessionCountKind> {
        if let Self::CountOverflow { counter, .. } = *self {
            Some(counter)
        } else {
            None
        }
    }
}
impl fmt::Display for CoverageSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidTestName { ref name } => {
                write!(f, "invalid coverage-session test name `{name}`")
            }
            Self::InvalidGroup {
                ref test,
                ref source,
            } => write!(
                f,
                "coverage session for test `{test}` could not capture coverage group: {source}"
            ),
            Self::DuplicateInstancePath {
                ref test,
                ref instance_path,
            } => write!(
                f,
                "coverage session for test `{test}` already contains instance path \
                 `{instance_path}`"
            ),
            Self::CountOverflow { ref test, counter } => write!(
                f,
                "coverage session for test `{test}` overflowed its {counter}"
            ),
        }
    }
}
impl std::error::Error for CoverageSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.group_error()
            .map(|source| -> &(dyn std::error::Error + 'static) { source })
    }
}
