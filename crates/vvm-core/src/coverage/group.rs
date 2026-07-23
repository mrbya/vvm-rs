use std::collections::BTreeSet;
use std::sync::Arc;

use super::identifier::{is_valid_coverage_identifier, is_valid_coverage_path};
use crate::coverage::{
    CoverageGroupCountKind, CoverageGroupError, CoverageItemKind, CoverageItemRef, CoverageRatio,
    CrossAxis,
};

/// Stable identity of one coverage-group instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoverageGroupInstance {
    /// Reusable group-definition name.
    definition_name: Arc<str>,
    /// Explicit semantic definition revision.
    definition_revision: u64,
    /// Hierarchical instance path.
    instance_path: Arc<str>,
}

impl CoverageGroupInstance {
    /// Constructs one validated group instance.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageGroupError`] when the definition name or instance path
    /// is invalid.
    pub fn new(
        definition_name: impl Into<String>,
        instance_path: impl Into<String>,
    ) -> Result<Self, CoverageGroupError> {
        Self::new_with_revision(definition_name, instance_path, 0)
    }

    /// Constructs one validated group instance with a semantic definition revision.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageGroupError`] when the definition name or instance path
    /// is invalid.
    pub fn new_with_revision(
        definition_name: impl Into<String>,
        instance_path: impl Into<String>,
        definition_revision: u64,
    ) -> Result<Self, CoverageGroupError> {
        let definition_name = definition_name.into();
        let instance_path = instance_path.into();
        if !is_valid_coverage_identifier(&definition_name) {
            return Err(CoverageGroupError::InvalidDefinitionName {
                name: definition_name,
            });
        }
        if !is_valid_coverage_path(&instance_path) {
            return Err(CoverageGroupError::InvalidInstancePath {
                path: instance_path,
            });
        }
        Ok(Self {
            definition_name: Arc::from(definition_name),
            definition_revision,
            instance_path: Arc::from(instance_path),
        })
    }
    /// Returns the reusable coverage definition name.
    #[must_use]
    pub fn definition_name(&self) -> &str {
        &self.definition_name
    }
    /// Returns the semantic definition revision.
    #[must_use]
    pub const fn definition_revision(&self) -> u64 {
        self.definition_revision
    }
    /// Returns the hierarchical instance path.
    #[must_use]
    pub fn instance_path(&self) -> &str {
        &self.instance_path
    }
}

/// Visitor receiving coverage items in group-definition order.
pub trait CoverageGroupVisitor {
    /// Visits one coverage item.
    fn visit(&mut self, item: CoverageItemRef<'_>);
}

/// User-owned typed functional coverage group.
pub trait CoverageGroup {
    /// Returns this group's stable instance identity.
    fn instance(&self) -> &CoverageGroupInstance;
    /// Visits every coverage item in stable definition order.
    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor);
    /// Validates group structure.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid group structure.
    fn validate(&self) -> Result<(), CoverageGroupError> {
        validate_group(self)
    }
    /// Computes one exact group summary.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid structure or aggregate-count overflow.
    fn summary(&self) -> Result<CoverageGroupSummary, CoverageGroupError> {
        summarize_group(self)
    }
    /// Returns raw aggregate bin coverage.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid structure or aggregate-count overflow.
    fn coverage(&self) -> Result<CoverageRatio, CoverageGroupError> {
        Ok(self.summary()?.coverage())
    }
}

/// Exact read-only summary of one coverage-group instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageGroupSummary {
    /// Total exposed coverage items.
    items: usize,
    /// Total typed coverpoints.
    coverpoints: usize,
    /// Total two-way crosses.
    crosses: usize,
    /// Aggregate exact bin coverage.
    coverage: CoverageRatio,
}

impl CoverageGroupSummary {
    /// Reconstructs a persisted group summary.
    pub(crate) const fn from_parts(
        items: usize,
        coverpoints: usize,
        crosses: usize,
        coverage: CoverageRatio,
    ) -> Self {
        Self {
            items,
            coverpoints,
            crosses,
            coverage,
        }
    }
    /// Returns total item count.
    #[must_use]
    pub const fn item_count(self) -> usize {
        self.items
    }
    /// Returns typed coverpoint count.
    #[must_use]
    pub const fn coverpoint_count(self) -> usize {
        self.coverpoints
    }
    /// Returns two-way cross count.
    #[must_use]
    pub const fn cross_count(self) -> usize {
        self.crosses
    }
    /// Returns exact aggregate bin coverage.
    #[must_use]
    pub const fn coverage(self) -> CoverageRatio {
        self.coverage
    }
    /// Returns whether all bins in all items are covered.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.coverage.is_complete()
    }
}

/// First-pass structural validation state.
struct ValidationCollector {
    /// Seen item names.
    names: BTreeSet<String>,
    /// Exact exposed coverpoint identities.
    coverpoints: Vec<Arc<str>>,
    /// Cross names and exact axis identities.
    crosses: Vec<(String, Arc<str>, Arc<str>)>,
    /// Number of visited items.
    items: usize,
    /// First deterministic validation error.
    error: Option<CoverageGroupError>,
    /// Group definition name for diagnostics.
    definition: String,
    /// Group instance path for diagnostics.
    instance: String,
}

impl CoverageGroupVisitor for ValidationCollector {
    fn visit(&mut self, item: CoverageItemRef<'_>) {
        if self.error.is_some() {
            return;
        }
        let Some(next_items) = self.items.checked_add(1) else {
            self.error = Some(CoverageGroupError::CountOverflow {
                definition: self.definition.clone(),
                instance: self.instance.clone(),
                counter: CoverageGroupCountKind::Items,
            });
            return;
        };
        self.items = next_items;
        if !self.names.insert(item.name().to_owned()) {
            self.error = Some(CoverageGroupError::DuplicateItemName {
                definition: self.definition.clone(),
                instance: self.instance.clone(),
                item: item.name().to_owned(),
            });
            return;
        }
        if let Some(source) = item.coverpoint_source_identity() {
            if self
                .coverpoints
                .iter()
                .any(|existing| Arc::ptr_eq(existing, source))
            {
                self.error = Some(CoverageGroupError::DuplicateCoverpoint {
                    definition: self.definition.clone(),
                    instance: self.instance.clone(),
                    coverpoint: item.name().to_owned(),
                });
                return;
            }
            self.coverpoints.push(Arc::clone(source));
        }
        if let Some((left, right)) = item.cross_source_identities() {
            self.crosses
                .push((item.name().to_owned(), Arc::clone(left), Arc::clone(right)));
        }
    }
}

/// Validates one group definition.
fn validate_group<G>(group: &G) -> Result<(), CoverageGroupError>
where
    G: CoverageGroup + ?Sized,
{
    let instance = group.instance();
    let definition = instance.definition_name().to_owned();
    let path = instance.instance_path().to_owned();
    let mut collector = ValidationCollector {
        names: BTreeSet::new(),
        coverpoints: Vec::new(),
        crosses: Vec::new(),
        items: 0,
        error: None,
        definition: definition.clone(),
        instance: path.clone(),
    };
    group.visit_items(&mut collector);
    if let Some(error) = collector.error {
        return Err(error);
    }
    if collector.items == 0 {
        return Err(CoverageGroupError::EmptyGroup {
            definition,
            instance: path,
        });
    }
    for (cross, left, right) in collector.crosses {
        if !collector
            .coverpoints
            .iter()
            .any(|source| Arc::ptr_eq(source, &left))
        {
            return Err(CoverageGroupError::MissingCrossSource {
                definition,
                instance: path,
                cross,
                axis: CrossAxis::Left,
                coverpoint: left.to_string(),
            });
        }
        if !collector
            .coverpoints
            .iter()
            .any(|source| Arc::ptr_eq(source, &right))
        {
            return Err(CoverageGroupError::MissingCrossSource {
                definition,
                instance: path,
                cross,
                axis: CrossAxis::Right,
                coverpoint: right.to_string(),
            });
        }
    }
    Ok(())
}

/// Computes one checked aggregate group summary.
fn summarize_group<G>(group: &G) -> Result<CoverageGroupSummary, CoverageGroupError>
where
    G: CoverageGroup + ?Sized,
{
    validate_group(group)?;
    let instance = group.instance();
    let mut collector = SummaryCollector {
        items: 0,
        coverpoints: 0,
        crosses: 0,
        covered: 0,
        uncovered: 0,
        total: 0,
        error: None,
        instance,
    };
    group.visit_items(&mut collector);
    if let Some(error) = collector.error {
        return Err(error);
    }
    Ok(CoverageGroupSummary {
        items: collector.items,
        coverpoints: collector.coverpoints,
        crosses: collector.crosses,
        coverage: CoverageRatio::new(collector.covered, collector.uncovered, collector.total),
    })
}

/// Checked aggregate summary state.
struct SummaryCollector<'a> {
    /// Number of visited items.
    items: usize,
    /// Number of typed coverpoints.
    coverpoints: usize,
    /// Number of two-way crosses.
    crosses: usize,
    /// Aggregate covered bins.
    covered: usize,
    /// Aggregate uncovered bins.
    uncovered: usize,
    /// Aggregate total bins.
    total: usize,
    /// First aggregate-count error.
    error: Option<CoverageGroupError>,
    /// Group identity used for overflow diagnostics.
    instance: &'a CoverageGroupInstance,
}
impl CoverageGroupVisitor for SummaryCollector<'_> {
    fn visit(&mut self, item: CoverageItemRef<'_>) {
        if self.error.is_some() {
            return;
        }
        let result = (|| {
            self.items = checked_add(self.instance, self.items, 1, CoverageGroupCountKind::Items)?;
            match item.kind() {
                CoverageItemKind::Coverpoint => {
                    self.coverpoints = checked_add(
                        self.instance,
                        self.coverpoints,
                        1,
                        CoverageGroupCountKind::Coverpoints,
                    )?;
                }
                CoverageItemKind::Cross2 => {
                    self.crosses = checked_add(
                        self.instance,
                        self.crosses,
                        1,
                        CoverageGroupCountKind::Crosses,
                    )?;
                }
            }
            let ratio = item.coverage();
            self.covered = checked_add(
                self.instance,
                self.covered,
                ratio.covered(),
                CoverageGroupCountKind::CoveredBins,
            )?;
            self.uncovered = checked_add(
                self.instance,
                self.uncovered,
                ratio.uncovered(),
                CoverageGroupCountKind::UncoveredBins,
            )?;
            self.total = checked_add(
                self.instance,
                self.total,
                ratio.total(),
                CoverageGroupCountKind::TotalBins,
            )?;
            Ok(())
        })();
        if let Err(error) = result {
            self.error = Some(error);
        }
    }
}

/// Adds one aggregate count with structured overflow context.
fn checked_add(
    instance: &CoverageGroupInstance,
    current: usize,
    added: usize,
    counter: CoverageGroupCountKind,
) -> Result<usize, CoverageGroupError> {
    current
        .checked_add(added)
        .ok_or_else(|| CoverageGroupError::CountOverflow {
            definition: instance.definition_name().to_owned(),
            instance: instance.instance_path().to_owned(),
            counter,
        })
}
