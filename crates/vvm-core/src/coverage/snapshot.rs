use std::sync::Arc;

use crate::coverage::{
    BinId, BinKind, CoverageGroup, CoverageGroupError, CoverageGroupSummary, CoverageGroupVisitor,
    CoverageItemKind, CoverageItemRef, CoverageRatio, Coverpoint, Cross2, CrossBinId,
};

/// Immutable runtime snapshot of one coverpoint bin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverpointBinSnapshot {
    /// Coverpoint-local bin identifier.
    id: BinId,
    /// Stable bin name.
    name: Arc<str>,
    /// Normal, ignore, or illegal role.
    kind: BinKind,
    /// Captured hit count.
    hits: u64,
    /// Required hit count.
    required_hits: u64,
}

impl CoverpointBinSnapshot {
    /// Returns the coverpoint-local bin identifier.
    #[must_use]
    pub const fn id(&self) -> BinId {
        self.id
    }
    /// Returns the stable bin name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the normal, ignore, or illegal role.
    #[must_use]
    pub const fn kind(&self) -> BinKind {
        self.kind
    }
    /// Returns the captured hit count.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }
    /// Returns the required hit count.
    #[must_use]
    pub const fn required_hits(&self) -> u64 {
        self.required_hits
    }
    /// Returns whether this normal bin is covered.
    #[must_use]
    pub const fn covered(&self) -> bool {
        matches!(self.kind, BinKind::Normal) && self.hits >= self.required_hits
    }
}

/// Immutable runtime snapshot of one generated two-way cross bin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossBinSnapshot {
    /// Cross-local bin identifier.
    id: CrossBinId,
    /// Left source-bin identifier.
    left_bin_id: BinId,
    /// Left source-bin name.
    left_bin_name: Arc<str>,
    /// Right source-bin identifier.
    right_bin_id: BinId,
    /// Right source-bin name.
    right_bin_name: Arc<str>,
    /// Captured hit count.
    hits: u64,
    /// Required hit count.
    required_hits: u64,
}

impl CrossBinSnapshot {
    /// Returns the cross-local bin identifier.
    #[must_use]
    pub const fn id(&self) -> CrossBinId {
        self.id
    }
    /// Returns the left source-bin identifier.
    #[must_use]
    pub const fn left_bin_id(&self) -> BinId {
        self.left_bin_id
    }
    /// Returns the left source-bin name.
    #[must_use]
    pub fn left_bin_name(&self) -> &str {
        &self.left_bin_name
    }
    /// Returns the right source-bin identifier.
    #[must_use]
    pub const fn right_bin_id(&self) -> BinId {
        self.right_bin_id
    }
    /// Returns the right source-bin name.
    #[must_use]
    pub fn right_bin_name(&self) -> &str {
        &self.right_bin_name
    }
    /// Returns the captured hit count.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }
    /// Returns the required hit count.
    #[must_use]
    pub const fn required_hits(&self) -> u64 {
        self.required_hits
    }
    /// Returns whether the cross bin is covered.
    #[must_use]
    pub const fn covered(&self) -> bool {
        self.hits >= self.required_hits
    }
}

/// Immutable type-erased runtime snapshot of one typed coverpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverpointSnapshot {
    /// Coverpoint name.
    name: Arc<str>,
    /// Bins in declaration order.
    bins: Box<[CoverpointBinSnapshot]>,
    /// Total attempted samples.
    samples: u64,
    /// Ignored samples.
    ignored_samples: u64,
    /// Illegal samples.
    illegal_samples: u64,
    /// Unmatched samples.
    unmatched_samples: u64,
    /// Exact captured normal-bin coverage.
    coverage: CoverageRatio,
}

impl CoverpointSnapshot {
    /// Captures an owned snapshot of one live coverpoint.
    pub(crate) fn capture<T>(coverpoint: &Coverpoint<T>) -> Self {
        let bins = coverpoint
            .bins()
            .iter()
            .map(|bin| CoverpointBinSnapshot {
                id: bin.id(),
                name: Arc::from(bin.name()),
                kind: bin.kind(),
                hits: bin.hits(),
                required_hits: bin.required_hits(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            name: Arc::from(coverpoint.name()),
            bins,
            samples: coverpoint.sample_count(),
            ignored_samples: coverpoint.ignored_sample_count(),
            illegal_samples: coverpoint.illegal_sample_count(),
            unmatched_samples: coverpoint.unmatched_sample_count(),
            coverage: coverpoint.coverage(),
        }
    }
    /// Returns the coverpoint name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns bins in declaration order.
    #[must_use]
    pub fn bins(&self) -> &[CoverpointBinSnapshot] {
        &self.bins
    }
    /// Returns total attempted samples.
    #[must_use]
    pub const fn sample_count(&self) -> u64 {
        self.samples
    }
    /// Returns ignored samples.
    #[must_use]
    pub const fn ignored_sample_count(&self) -> u64 {
        self.ignored_samples
    }
    /// Returns illegal samples.
    #[must_use]
    pub const fn illegal_sample_count(&self) -> u64 {
        self.illegal_samples
    }
    /// Returns unmatched samples.
    #[must_use]
    pub const fn unmatched_sample_count(&self) -> u64 {
        self.unmatched_samples
    }
    /// Returns exact captured normal-bin coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.coverage
    }
    /// Iterates covered normal bins in declaration order.
    pub fn covered_bins(&self) -> impl Iterator<Item = &CoverpointBinSnapshot> {
        self.bins.iter().filter(|bin| bin.covered())
    }
    /// Iterates uncovered normal bins in declaration order.
    pub fn uncovered_bins(&self) -> impl Iterator<Item = &CoverpointBinSnapshot> {
        self.bins
            .iter()
            .filter(|bin| bin.kind() == BinKind::Normal && !bin.covered())
    }
}

/// Immutable runtime snapshot of one two-way cross.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cross2Snapshot {
    /// Cross name.
    name: Arc<str>,
    /// Left source coverpoint name.
    left_coverpoint_name: Arc<str>,
    /// Right source coverpoint name.
    right_coverpoint_name: Arc<str>,
    /// Generated bins in row-major order.
    bins: Box<[CrossBinSnapshot]>,
    /// Total cross sample calls.
    samples: u64,
    /// Skipped cross samples.
    skipped_samples: u64,
    /// Exact captured cross-bin coverage.
    coverage: CoverageRatio,
}

impl Cross2Snapshot {
    /// Captures an owned snapshot of one live cross.
    pub(crate) fn capture(cross: &Cross2) -> Self {
        let bins = cross
            .bins()
            .iter()
            .map(|bin| CrossBinSnapshot {
                id: bin.id(),
                left_bin_id: bin.left_bin_id(),
                left_bin_name: Arc::from(bin.left_bin_name()),
                right_bin_id: bin.right_bin_id(),
                right_bin_name: Arc::from(bin.right_bin_name()),
                hits: bin.hits(),
                required_hits: bin.required_hits(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            name: Arc::from(cross.name()),
            left_coverpoint_name: Arc::from(cross.left_coverpoint_name()),
            right_coverpoint_name: Arc::from(cross.right_coverpoint_name()),
            bins,
            samples: cross.sample_count(),
            skipped_samples: cross.skipped_sample_count(),
            coverage: cross.coverage(),
        }
    }
    /// Returns the cross name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the left source coverpoint name.
    #[must_use]
    pub fn left_coverpoint_name(&self) -> &str {
        &self.left_coverpoint_name
    }
    /// Returns the right source coverpoint name.
    #[must_use]
    pub fn right_coverpoint_name(&self) -> &str {
        &self.right_coverpoint_name
    }
    /// Returns generated bins in row-major order.
    #[must_use]
    pub fn bins(&self) -> &[CrossBinSnapshot] {
        &self.bins
    }
    /// Returns total cross sample calls.
    #[must_use]
    pub const fn sample_count(&self) -> u64 {
        self.samples
    }
    /// Returns skipped cross sample calls.
    #[must_use]
    pub const fn skipped_sample_count(&self) -> u64 {
        self.skipped_samples
    }
    /// Returns exact captured cross-bin coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.coverage
    }
    /// Iterates covered cross bins in row-major order.
    pub fn covered_bins(&self) -> impl Iterator<Item = &CrossBinSnapshot> {
        self.bins.iter().filter(|bin| bin.covered())
    }
    /// Iterates uncovered cross bins in row-major order.
    pub fn uncovered_bins(&self) -> impl Iterator<Item = &CrossBinSnapshot> {
        self.bins.iter().filter(|bin| !bin.covered())
    }
}

/// Immutable snapshot of one supported coverage item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverageItemSnapshot {
    /// Typed coverpoint snapshot.
    Coverpoint(CoverpointSnapshot),
    /// Two-way cross snapshot.
    Cross2(Cross2Snapshot),
}

impl CoverageItemSnapshot {
    /// Returns the item kind.
    #[must_use]
    pub const fn kind(&self) -> CoverageItemKind {
        if self.as_coverpoint().is_some() {
            CoverageItemKind::Coverpoint
        } else {
            CoverageItemKind::Cross2
        }
    }
    /// Returns the item name.
    #[must_use]
    pub fn name(&self) -> &str {
        match *self {
            Self::Coverpoint(ref item) => item.name(),
            Self::Cross2(ref item) => item.name(),
        }
    }
    /// Returns exact coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        match *self {
            Self::Coverpoint(ref item) => item.coverage(),
            Self::Cross2(ref item) => item.coverage(),
        }
    }
    /// Returns sample calls.
    #[must_use]
    pub const fn sample_count(&self) -> u64 {
        match *self {
            Self::Coverpoint(ref item) => item.sample_count(),
            Self::Cross2(ref item) => item.sample_count(),
        }
    }
    /// Returns the coverpoint snapshot when applicable.
    #[must_use]
    pub const fn as_coverpoint(&self) -> Option<&CoverpointSnapshot> {
        if let Self::Coverpoint(ref item) = *self {
            Some(item)
        } else {
            None
        }
    }
    /// Returns the cross snapshot when applicable.
    #[must_use]
    pub const fn as_cross2(&self) -> Option<&Cross2Snapshot> {
        if let Self::Cross2(ref item) = *self {
            Some(item)
        } else {
            None
        }
    }
}

/// Immutable snapshot of one coverage-group instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageGroupSnapshot {
    /// Reusable definition name.
    definition_name: Arc<str>,
    /// Hierarchical instance path.
    instance_path: Arc<str>,
    /// Items in group visitation order.
    items: Box<[CoverageItemSnapshot]>,
    /// Exact captured group summary.
    summary: CoverageGroupSummary,
}

impl CoverageGroupSnapshot {
    /// Returns the reusable definition name.
    #[must_use]
    pub fn definition_name(&self) -> &str {
        &self.definition_name
    }
    /// Returns the hierarchical instance path.
    #[must_use]
    pub fn instance_path(&self) -> &str {
        &self.instance_path
    }
    /// Returns items in visitation order.
    #[must_use]
    pub fn items(&self) -> &[CoverageItemSnapshot] {
        &self.items
    }
    /// Returns the exact group summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageGroupSummary {
        self.summary
    }
    /// Returns exact group coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.summary.coverage()
    }
    /// Finds an item by name.
    #[must_use]
    pub fn item(&self, name: &str) -> Option<&CoverageItemSnapshot> {
        self.items.iter().find(|item| item.name() == name)
    }
}

/// Collects owned item snapshots in group visitation order.
struct GroupSnapshotCollector {
    /// Captured items in visitation order.
    items: Vec<CoverageItemSnapshot>,
}
impl CoverageGroupVisitor for GroupSnapshotCollector {
    fn visit(&mut self, item: CoverageItemRef<'_>) {
        self.items.push(item.snapshot());
    }
}

/// Captures one validated group into an owned snapshot.
pub(super) fn capture_group<G>(group: &G) -> Result<CoverageGroupSnapshot, CoverageGroupError>
where
    G: CoverageGroup + ?Sized,
{
    let summary = group.summary()?;
    let instance = group.instance();
    let mut collector = GroupSnapshotCollector { items: Vec::new() };

    group.visit_items(&mut collector);

    Ok(CoverageGroupSnapshot {
        definition_name: Arc::from(instance.definition_name()),
        instance_path: Arc::from(instance.instance_path()),
        items: collector.items.into_boxed_slice(),
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::capture_group;
    use crate::{
        Bin, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor, CoverageItemRef,
        Coverpoint,
    };

    struct Group {
        instance: CoverageGroupInstance,
        point: Coverpoint<u8>,
    }

    impl CoverageGroup for Group {
        fn instance(&self) -> &CoverageGroupInstance {
            &self.instance
        }

        fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
            visitor.visit(CoverageItemRef::coverpoint(&self.point));
        }
    }

    fn group() -> Result<Group, Box<dyn std::error::Error>> {
        Ok(Group {
            instance: CoverageGroupInstance::new("decoder", "dut.decoder")?,
            point: Coverpoint::builder("opcode")
                .bin(Bin::value("read", 1_u8))
                .bin(Bin::value("write", 2_u8))
                .build()?,
        })
    }

    #[test]
    fn captures_coverpoint_bins_in_declaration_order() -> Result<(), Box<dyn std::error::Error>> {
        let group = group()?;
        let snapshot = capture_group(&group)?;
        let items = snapshot.items();
        let point = items
            .first()
            .and_then(crate::CoverageItemSnapshot::as_coverpoint)
            .ok_or("missing coverpoint")?;
        let names = point
            .bins()
            .iter()
            .map(crate::CoverpointBinSnapshot::name)
            .collect::<Vec<_>>();

        assert_eq!(names, ["read", "write"]);
        assert_eq!(point.coverage().total(), 2);
        Ok(())
    }

    #[test]
    fn snapshot_does_not_change_after_later_coverpoint_sampling()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut group = group()?;
        let snapshot = capture_group(&group)?;

        group.point.sample(&1)?;

        let point = snapshot
            .items()
            .first()
            .and_then(crate::CoverageItemSnapshot::as_coverpoint)
            .ok_or("missing coverpoint")?;
        assert_eq!(point.sample_count(), 0);
        assert_eq!(point.coverage().covered(), 0);
        Ok(())
    }
}
