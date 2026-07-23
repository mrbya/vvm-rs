use std::sync::Arc;

use crate::coverage::{
    CoverageItemSnapshot, CoverageRatio, Coverpoint, CoverpointSnapshot, Cross2, Cross2Snapshot,
};

/// Internal read-only view over any typed coverpoint.
trait ErasedCoverpoint {
    /// Returns the coverpoint name.
    fn item_name(&self) -> &str;
    /// Returns exact process-local source identity.
    fn item_source_identity(&self) -> &Arc<str>;
    /// Returns exact normal-bin coverage.
    fn item_coverage(&self) -> CoverageRatio;
    /// Returns attempted sample count.
    fn item_sample_count(&self) -> u64;
    /// Captures one owned immutable snapshot.
    fn item_snapshot(&self) -> CoverpointSnapshot;
}

impl<T> ErasedCoverpoint for Coverpoint<T> {
    fn item_name(&self) -> &str {
        self.name()
    }
    fn item_source_identity(&self) -> &Arc<str> {
        self.source_identity()
    }
    fn item_coverage(&self) -> CoverageRatio {
        self.coverage()
    }
    fn item_sample_count(&self) -> u64 {
        self.sample_count()
    }
    fn item_snapshot(&self) -> CoverpointSnapshot {
        CoverpointSnapshot::capture(self)
    }
}

/// Kind of one item exposed by a coverage group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageItemKind {
    /// Typed functional coverpoint.
    Coverpoint,
    /// Complete two-way cross.
    Cross2,
}

impl std::fmt::Display for CoverageItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Coverpoint => f.write_str("coverpoint"),
            Self::Cross2 => f.write_str("cross"),
        }
    }
}

/// Read-only type-erased reference to one supported coverage item.
#[derive(Clone, Copy)]
pub struct CoverageItemRef<'a> {
    /// Private erased item representation.
    inner: CoverageItemRefKind<'a>,
}

/// Supported read-only item representations.
#[derive(Clone, Copy)]
enum CoverageItemRefKind<'a> {
    /// Typed coverpoint inspection.
    Coverpoint(&'a dyn ErasedCoverpoint),
    /// Two-way cross inspection.
    Cross2(&'a Cross2),
}

impl<'a> CoverageItemRef<'a> {
    /// Creates a read-only item view over a typed coverpoint.
    #[must_use]
    pub fn coverpoint<T>(coverpoint: &'a Coverpoint<T>) -> Self {
        Self {
            inner: CoverageItemRefKind::Coverpoint(coverpoint),
        }
    }
    /// Creates a read-only item view over a two-way cross.
    #[must_use]
    pub const fn cross2(cross: &'a Cross2) -> Self {
        Self {
            inner: CoverageItemRefKind::Cross2(cross),
        }
    }
    /// Returns the item kind.
    #[must_use]
    pub const fn kind(self) -> CoverageItemKind {
        match self.inner {
            CoverageItemRefKind::Coverpoint(_) => CoverageItemKind::Coverpoint,
            CoverageItemRefKind::Cross2(_) => CoverageItemKind::Cross2,
        }
    }
    /// Returns the item name.
    #[must_use]
    pub fn name(self) -> &'a str {
        match self.inner {
            CoverageItemRefKind::Coverpoint(item) => item.item_name(),
            CoverageItemRefKind::Cross2(item) => item.name(),
        }
    }
    /// Returns exact bin coverage.
    #[must_use]
    pub fn coverage(self) -> CoverageRatio {
        match self.inner {
            CoverageItemRefKind::Coverpoint(item) => item.item_coverage(),
            CoverageItemRefKind::Cross2(item) => item.coverage(),
        }
    }
    /// Returns the item's sample-call count.
    #[must_use]
    pub fn sample_count(self) -> u64 {
        match self.inner {
            CoverageItemRefKind::Coverpoint(item) => item.item_sample_count(),
            CoverageItemRefKind::Cross2(item) => item.sample_count(),
        }
    }
    /// Returns a coverpoint's exact process-local source identity.
    pub(crate) fn coverpoint_source_identity(self) -> Option<&'a Arc<str>> {
        match self.inner {
            CoverageItemRefKind::Coverpoint(item) => Some(item.item_source_identity()),
            CoverageItemRefKind::Cross2(_) => None,
        }
    }
    /// Returns exact left and right source identities for a cross.
    pub(crate) const fn cross_source_identities(self) -> Option<(&'a Arc<str>, &'a Arc<str>)> {
        match self.inner {
            CoverageItemRefKind::Coverpoint(_) => None,
            CoverageItemRefKind::Cross2(item) => {
                Some((item.left_source_identity(), item.right_source_identity()))
            }
        }
    }
    /// Captures an immutable owned snapshot.
    pub(crate) fn snapshot(self) -> CoverageItemSnapshot {
        match self.inner {
            CoverageItemRefKind::Coverpoint(item) => {
                CoverageItemSnapshot::Coverpoint(item.item_snapshot())
            }
            CoverageItemRefKind::Cross2(item) => {
                CoverageItemSnapshot::Cross2(Cross2Snapshot::capture(item))
            }
        }
    }
}
