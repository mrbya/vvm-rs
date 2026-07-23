use std::collections::BTreeSet;
use std::sync::Arc;

use super::snapshot::capture_group;
use crate::coverage::{
    CoverageGroup, CoverageGroupSnapshot, CoverageRatio, CoverageSessionCountKind,
    CoverageSessionError,
};
use crate::registry::is_valid_test_name;

/// Exact aggregate summary of one non-empty coverage session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageSessionSummary {
    /// Captured group instances.
    groups: usize,
    /// Captured coverage items.
    items: usize,
    /// Captured typed coverpoints.
    coverpoints: usize,
    /// Captured two-way crosses.
    crosses: usize,
    /// Exact flat aggregate coverage.
    coverage: CoverageRatio,
}
impl CoverageSessionSummary {
    /// Reconstructs a persisted session summary.
    pub(crate) const fn from_parts(
        groups: usize,
        items: usize,
        coverpoints: usize,
        crosses: usize,
        coverage: CoverageRatio,
    ) -> Self {
        Self {
            groups,
            items,
            coverpoints,
            crosses,
            coverage,
        }
    }
    /// Returns captured group instances.
    #[must_use]
    pub const fn group_count(self) -> usize {
        self.groups
    }
    /// Returns captured items.
    #[must_use]
    pub const fn item_count(self) -> usize {
        self.items
    }
    /// Returns captured coverpoints.
    #[must_use]
    pub const fn coverpoint_count(self) -> usize {
        self.coverpoints
    }
    /// Returns captured two-way crosses.
    #[must_use]
    pub const fn cross_count(self) -> usize {
        self.crosses
    }
    /// Returns flat aggregate coverage.
    #[must_use]
    pub const fn coverage(self) -> CoverageRatio {
        self.coverage
    }
    /// Returns whether all bins are covered.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.coverage.is_complete()
    }
}

/// Mutable per-test collector of immutable coverage-group snapshots.
pub struct CoverageSession {
    /// Stable VVM test name.
    test_name: Arc<str>,
    /// Captured groups in capture order.
    groups: Vec<CoverageGroupSnapshot>,
    /// Instance paths already captured.
    instance_paths: BTreeSet<String>,
    /// Aggregate summary for captured groups.
    summary: Option<CoverageSessionSummary>,
}
impl CoverageSession {
    /// Constructs an empty per-test coverage session.
    ///
    /// # Errors
    /// Returns [`CoverageSessionError::InvalidTestName`] for an invalid name.
    pub fn new(test_name: impl Into<String>) -> Result<Self, CoverageSessionError> {
        let test_name = test_name.into();
        if !is_valid_test_name(&test_name) {
            return Err(CoverageSessionError::InvalidTestName { name: test_name });
        }
        Ok(Self::new_validated(test_name))
    }
    /// Constructs a session after registry-name validation.
    pub(crate) fn new_validated(test_name: impl Into<String>) -> Self {
        Self {
            test_name: Arc::from(test_name.into()),
            groups: Vec::new(),
            instance_paths: BTreeSet::new(),
            summary: None,
        }
    }
    /// Returns the associated VVM test name.
    #[must_use]
    pub fn test_name(&self) -> &str {
        &self.test_name
    }
    /// Returns captured groups in capture order.
    #[must_use]
    pub fn groups(&self) -> &[CoverageGroupSnapshot] {
        &self.groups
    }
    /// Finds a group by instance path.
    #[must_use]
    pub fn group(&self, instance_path: &str) -> Option<&CoverageGroupSnapshot> {
        self.groups
            .iter()
            .find(|group| group.instance_path() == instance_path)
    }
    /// Returns whether no group has been captured.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
    /// Returns the current aggregate summary.
    #[must_use]
    pub const fn summary(&self) -> Option<CoverageSessionSummary> {
        self.summary
    }
    /// Captures one coverage-group instance, freezing its current state.
    ///
    /// # Errors
    /// Returns an error for an invalid group, duplicate path, or count overflow.
    pub fn capture<G>(&mut self, group: &G) -> Result<(), CoverageSessionError>
    where
        G: CoverageGroup + ?Sized,
    {
        let instance_path = group.instance().instance_path().to_owned();
        if self.instance_paths.contains(&instance_path) {
            return Err(CoverageSessionError::DuplicateInstancePath {
                test: self.test_name.to_string(),
                instance_path,
            });
        }

        let snapshot =
            capture_group(group).map_err(|source| CoverageSessionError::InvalidGroup {
                test: self.test_name.to_string(),
                source: Box::new(source),
            })?;
        let summary = next_summary(self.summary, snapshot.summary(), self.test_name())?;

        self.instance_paths.insert(instance_path);
        self.groups.push(snapshot);
        self.summary = Some(summary);
        Ok(())
    }
    /// Completes the session, returning no snapshot when no groups were captured.
    #[must_use]
    pub fn finish(self) -> Option<CoverageSessionSnapshot> {
        self.summary.map(|summary| CoverageSessionSnapshot {
            test_name: self.test_name,
            groups: self.groups.into_boxed_slice(),
            summary,
        })
    }
}

/// Immutable completed coverage data from one VVM test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageSessionSnapshot {
    /// Stable VVM test name.
    test_name: Arc<str>,
    /// Captured group instances in capture order.
    groups: Box<[CoverageGroupSnapshot]>,
    /// Exact aggregate summary.
    summary: CoverageSessionSummary,
}
impl CoverageSessionSnapshot {
    /// Returns the stable VVM test name.
    #[must_use]
    pub fn test_name(&self) -> &str {
        &self.test_name
    }
    /// Returns captured group instances in capture order.
    #[must_use]
    pub fn groups(&self) -> &[CoverageGroupSnapshot] {
        &self.groups
    }
    /// Finds a group by instance path.
    #[must_use]
    pub fn group(&self, instance_path: &str) -> Option<&CoverageGroupSnapshot> {
        self.groups
            .iter()
            .find(|group| group.instance_path() == instance_path)
    }
    /// Returns the exact aggregate summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageSessionSummary {
        self.summary
    }
    /// Returns flat aggregate coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.summary.coverage()
    }

    /// Consumes the snapshot into its test name, groups, and summary.
    pub(crate) fn into_parts(
        self,
    ) -> (
        Arc<str>,
        Box<[CoverageGroupSnapshot]>,
        CoverageSessionSummary,
    ) {
        (self.test_name, self.groups, self.summary)
    }
}

/// Calculates the next session summary without mutating the session.
fn next_summary(
    current: Option<CoverageSessionSummary>,
    group: crate::CoverageGroupSummary,
    test: &str,
) -> Result<CoverageSessionSummary, CoverageSessionError> {
    let current = current.unwrap_or(CoverageSessionSummary {
        groups: 0,
        items: 0,
        coverpoints: 0,
        crosses: 0,
        coverage: CoverageRatio::new(0, 0, 0),
    });
    let covered = checked_add(
        current.coverage.covered(),
        group.coverage().covered(),
        test,
        CoverageSessionCountKind::CoveredBins,
    )?;
    let uncovered = checked_add(
        current.coverage.uncovered(),
        group.coverage().uncovered(),
        test,
        CoverageSessionCountKind::UncoveredBins,
    )?;
    let total = checked_add(
        current.coverage.total(),
        group.coverage().total(),
        test,
        CoverageSessionCountKind::TotalBins,
    )?;
    Ok(CoverageSessionSummary {
        groups: checked_add(current.groups, 1, test, CoverageSessionCountKind::Groups)?,
        items: checked_add(
            current.items,
            group.item_count(),
            test,
            CoverageSessionCountKind::Items,
        )?,
        coverpoints: checked_add(
            current.coverpoints,
            group.coverpoint_count(),
            test,
            CoverageSessionCountKind::Coverpoints,
        )?,
        crosses: checked_add(
            current.crosses,
            group.cross_count(),
            test,
            CoverageSessionCountKind::Crosses,
        )?,
        coverage: CoverageRatio::new(covered, uncovered, total),
    })
}
/// Adds session counters with a structured overflow error.
fn checked_add(
    left: usize,
    right: usize,
    test: &str,
    counter: CoverageSessionCountKind,
) -> Result<usize, CoverageSessionError> {
    left.checked_add(right)
        .ok_or_else(|| CoverageSessionError::CountOverflow {
            test: test.to_owned(),
            counter,
        })
}

#[cfg(test)]
mod tests {
    use super::CoverageSession;
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
    fn group(path: &str) -> Result<Group, Box<dyn std::error::Error>> {
        Ok(Group {
            instance: CoverageGroupInstance::new("decoder", path)?,
            point: Coverpoint::builder("opcode")
                .bin(Bin::value("read", 1_u8))
                .build()?,
        })
    }

    #[test]
    fn capture_freezes_group_state_and_preserves_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut first = group("dut.first")?;
        let second = group("dut.second")?;
        let mut session = CoverageSession::new("decoder-random")?;
        session.capture(&first)?;
        session.capture(&second)?;
        first.point.sample(&1)?;

        assert_eq!(
            session
                .groups()
                .iter()
                .map(crate::CoverageGroupSnapshot::instance_path)
                .collect::<Vec<_>>(),
            ["dut.first", "dut.second"]
        );
        let first_snapshot = session.groups().first().ok_or("missing first group")?;
        assert_eq!(first_snapshot.coverage().covered(), 0);
        assert_eq!(session.summary().ok_or("missing summary")?.group_count(), 2);
        Ok(())
    }

    #[test]
    fn duplicate_capture_leaves_session_unchanged() -> Result<(), Box<dyn std::error::Error>> {
        let group = group("dut.decoder")?;
        let mut session = CoverageSession::new("decoder-random")?;
        session.capture(&group)?;
        let before = session.groups().to_vec();

        assert!(session.capture(&group).is_err());
        assert_eq!(session.groups(), before);
        Ok(())
    }
}
