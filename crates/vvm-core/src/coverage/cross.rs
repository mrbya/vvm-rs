use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::Arc;

use super::identifier::is_valid_coverage_identifier;
use crate::coverage::{
    BinId, CoverageRatio, CoverageSampleDisposition, Coverpoint, CoverpointSample, CrossAxis,
    CrossBuildError, CrossCounterKind, CrossSampleError,
};

/// Opaque identifier of one bin within its owning cross.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrossBinId(u32);

impl CrossBinId {
    /// Creates a cross-local bin identifier.
    pub(crate) const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// Returns the cross-local ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }
}

/// One normal bin captured from a source coverpoint.
struct CrossAxisBin {
    /// Original coverpoint-local bin ID.
    id: BinId,
    /// Stable bin name.
    name: Arc<str>,
}

/// One captured coverpoint axis.
struct CrossAxisDefinition {
    /// Exact source identity and coverpoint name.
    source: Arc<str>,
    /// Normal bins in declaration order.
    bins: Box<[CrossAxisBin]>,
}

impl CrossAxisDefinition {
    /// Captures one coverpoint definition.
    fn from_coverpoint<T>(coverpoint: &Coverpoint<T>) -> Self {
        let bins = coverpoint
            .normal_bins()
            .map(|bin| CrossAxisBin {
                id: bin.id(),
                name: Arc::<str>::from(bin.name()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            source: Arc::clone(coverpoint.source_identity()),
            bins,
        }
    }

    /// Returns the source coverpoint name.
    fn name(&self) -> &str {
        &self.source
    }

    /// Returns whether this definition contains an exact source-bin ID.
    fn contains(&self, id: BinId) -> bool {
        self.bins.iter().any(|bin| bin.id == id)
    }
}

/// Builder for one complete two-way normal-bin cross.
pub struct Cross2Builder {
    /// Stable cross name.
    name: String,
    /// Captured left axis.
    left: CrossAxisDefinition,
    /// Captured right axis.
    right: CrossAxisDefinition,
    /// Hit threshold applied to every generated cross bin.
    required_hits: u64,
    /// Maximum generated cross-bin count.
    max_bins: usize,
}

impl Cross2Builder {
    /// Sets the hit threshold applied to every generated cross bin.
    #[must_use]
    pub const fn at_least(mut self, required_hits: u64) -> Self {
        self.required_hits = required_hits;
        self
    }

    /// Overrides the maximum generated cross-bin count.
    #[must_use]
    pub const fn max_bins(mut self, max_bins: usize) -> Self {
        self.max_bins = max_bins;
        self
    }

    /// Validates and constructs the cross.
    ///
    /// # Errors
    ///
    /// Returns [`CrossBuildError`] for an invalid name, threshold, cardinality,
    /// or limit.
    pub fn build(self) -> Result<Cross2, CrossBuildError> {
        build_cross(self)
    }
}

/// One generated bin in a complete two-way cross.
pub struct CrossBin {
    /// Cross-local ID.
    id: CrossBinId,
    /// Left source-bin ID.
    left_bin_id: BinId,
    /// Left source-bin name.
    left_bin_name: Arc<str>,
    /// Right source-bin ID.
    right_bin_id: BinId,
    /// Right source-bin name.
    right_bin_name: Arc<str>,
    /// Required hit count.
    required_hits: NonZeroU64,
    /// Current hit count.
    hits: u64,
}

impl CrossBin {
    /// Returns the cross-local ID.
    #[must_use]
    pub const fn id(&self) -> CrossBinId {
        self.id
    }
    /// Returns the left source-bin ID.
    #[must_use]
    pub const fn left_bin_id(&self) -> BinId {
        self.left_bin_id
    }
    /// Returns the left source-bin name.
    #[must_use]
    pub fn left_bin_name(&self) -> &str {
        &self.left_bin_name
    }
    /// Returns the right source-bin ID.
    #[must_use]
    pub const fn right_bin_id(&self) -> BinId {
        self.right_bin_id
    }
    /// Returns the right source-bin name.
    #[must_use]
    pub fn right_bin_name(&self) -> &str {
        &self.right_bin_name
    }
    /// Returns the current hit count.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }
    /// Returns the required hit count.
    #[must_use]
    pub const fn required_hits(&self) -> u64 {
        self.required_hits.get()
    }
    /// Returns whether the cross bin is covered.
    #[must_use]
    pub const fn covered(&self) -> bool {
        self.hits >= self.required_hits.get()
    }
}

/// Disposition of one valid two-way cross sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrossSampleDisposition {
    /// Both axes hit normal bins and cross bins were incremented.
    Hit,
    /// At least one axis was ignored or unmatched.
    Skipped,
}

/// Result of one successfully processed cross sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossSample {
    /// Cross disposition.
    disposition: CrossSampleDisposition,
    /// Left source disposition.
    left_disposition: CoverageSampleDisposition,
    /// Right source disposition.
    right_disposition: CoverageSampleDisposition,
    /// Matching cross-bin IDs in row-major order.
    matched_bins: Box<[CrossBinId]>,
}

impl CrossSample {
    /// Returns the cross disposition.
    #[must_use]
    pub const fn disposition(&self) -> CrossSampleDisposition {
        self.disposition
    }
    /// Returns the left coverpoint-sample disposition.
    #[must_use]
    pub const fn left_disposition(&self) -> CoverageSampleDisposition {
        self.left_disposition
    }
    /// Returns the right coverpoint-sample disposition.
    #[must_use]
    pub const fn right_disposition(&self) -> CoverageSampleDisposition {
        self.right_disposition
    }
    /// Returns matching cross-bin IDs.
    #[must_use]
    pub fn matched_bins(&self) -> &[CrossBinId] {
        &self.matched_bins
    }
    /// Returns whether cross bins were hit.
    #[must_use]
    pub const fn hit(&self) -> bool {
        matches!(self.disposition, CrossSampleDisposition::Hit)
    }
    /// Returns whether the cross sample was skipped.
    #[must_use]
    pub const fn skipped(&self) -> bool {
        matches!(self.disposition, CrossSampleDisposition::Skipped)
    }
}

/// Explicitly sampled complete two-way functional cross.
pub struct Cross2 {
    /// Stable cross name.
    name: String,
    /// Captured left axis.
    left: CrossAxisDefinition,
    /// Captured right axis.
    right: CrossAxisDefinition,
    /// Generated bins in row-major declaration order.
    bins: Vec<CrossBin>,
    /// Pair-to-cross-bin lookup.
    bin_lookup: BTreeMap<(BinId, BinId), CrossBinId>,
    /// Valid cross sample calls.
    samples: u64,
    /// Samples skipped because one axis did not hit normal bins.
    skipped_samples: u64,
}

impl Cross2 {
    /// Default maximum generated-bin count.
    pub const DEFAULT_BIN_LIMIT: usize = 4096;

    /// Begins constructing a complete two-way cross.
    #[must_use]
    pub fn builder<L, R>(
        name: impl Into<String>,
        left: &Coverpoint<L>,
        right: &Coverpoint<R>,
    ) -> Cross2Builder {
        Cross2Builder {
            name: name.into(),
            left: CrossAxisDefinition::from_coverpoint(left),
            right: CrossAxisDefinition::from_coverpoint(right),
            required_hits: 1,
            max_bins: Self::DEFAULT_BIN_LIMIT,
        }
    }

    /// Returns the cross name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact left source identity.
    pub(crate) const fn left_source_identity(&self) -> &Arc<str> {
        &self.left.source
    }

    /// Returns the exact right source identity.
    pub(crate) const fn right_source_identity(&self) -> &Arc<str> {
        &self.right.source
    }
    /// Returns the left coverpoint name.
    #[must_use]
    pub fn left_coverpoint_name(&self) -> &str {
        self.left.name()
    }
    /// Returns the right coverpoint name.
    #[must_use]
    pub fn right_coverpoint_name(&self) -> &str {
        self.right.name()
    }
    /// Returns cross bins in row-major order.
    #[must_use]
    pub fn bins(&self) -> &[CrossBin] {
        &self.bins
    }
    /// Finds one cross bin by ID.
    #[must_use]
    pub fn bin(&self, id: CrossBinId) -> Option<&CrossBin> {
        usize::try_from(id.ordinal())
            .ok()
            .and_then(|index| self.bins.get(index))
    }
    /// Returns valid cross sample calls.
    #[must_use]
    pub const fn sample_count(&self) -> u64 {
        self.samples
    }
    /// Returns skipped cross sample calls.
    #[must_use]
    pub const fn skipped_sample_count(&self) -> u64 {
        self.skipped_samples
    }
    /// Returns exact generated-bin coverage.
    #[must_use]
    pub fn coverage(&self) -> CoverageRatio {
        let covered = self.bins.iter().filter(|bin| bin.covered()).count();
        let uncovered = self.bins.iter().filter(|bin| !bin.covered()).count();
        CoverageRatio::new(covered, uncovered, self.bins.len())
    }
    /// Returns covered cross bins in row-major order.
    pub fn covered_bins(&self) -> impl Iterator<Item = &CrossBin> {
        self.bins.iter().filter(|bin| bin.covered())
    }
    /// Returns uncovered cross bins in row-major order.
    pub fn uncovered_bins(&self) -> impl Iterator<Item = &CrossBin> {
        self.bins.iter().filter(|bin| !bin.covered())
    }

    /// Samples one pair of processed coverpoint samples.
    ///
    /// # Errors
    ///
    /// Returns an error for source mismatches, unknown source-bin identifiers,
    /// or counter overflow.
    pub fn sample(
        &mut self,
        left: &CoverpointSample,
        right: &CoverpointSample,
    ) -> Result<CrossSample, CrossSampleError> {
        let prepared = self.prepare_sample(left, right)?;

        self.commit_sample(&prepared);

        Ok(prepared.outcome.into_sample())
    }

    /// Preflights one cross sample without mutation.
    fn prepare_sample(
        &self,
        left: &CoverpointSample,
        right: &CoverpointSample,
    ) -> Result<PreparedCrossSample, CrossSampleError> {
        self.validate_source(left, &self.left, CrossAxis::Left)?;
        self.validate_source(right, &self.right, CrossAxis::Right)?;
        self.validate_bins(left, &self.left, CrossAxis::Left)?;
        self.validate_bins(right, &self.right, CrossAxis::Right)?;

        let next_samples =
            checked_increment(&self.name, self.samples, CrossCounterKind::Samples, None)?;
        let hit = left.hit() && right.hit();

        if !hit {
            let next_skipped_samples = checked_increment(
                &self.name,
                self.skipped_samples,
                CrossCounterKind::SkippedSamples,
                None,
            )?;
            return Ok(PreparedCrossSample {
                next_samples,
                next_bin_hits: vec![None; self.bins.len()].into_boxed_slice(),
                outcome: PreparedCrossOutcome::Skipped {
                    next_skipped_samples,
                    left_disposition: left.disposition(),
                    right_disposition: right.disposition(),
                },
            });
        }

        debug_assert!(!left.matched_bins().is_empty() && !right.matched_bins().is_empty());
        let mut matched_bins = Vec::new();
        let mut next_bin_hits = vec![None; self.bins.len()];

        for left_id in left.matched_bins() {
            for right_id in right.matched_bins() {
                let Some(cross_id) = self.bin_lookup.get(&(*left_id, *right_id)).copied() else {
                    return Err(CrossSampleError::UnknownBin {
                        cross: self.name.clone(),
                        axis: CrossAxis::Left,
                        coverpoint: self.left.name().to_owned(),
                        bin: *left_id,
                    });
                };
                let Some(bin) = self.bin(cross_id) else {
                    return Err(CrossSampleError::UnknownBin {
                        cross: self.name.clone(),
                        axis: CrossAxis::Left,
                        coverpoint: self.left.name().to_owned(),
                        bin: *left_id,
                    });
                };
                let next = checked_increment(
                    &self.name,
                    bin.hits,
                    CrossCounterKind::BinHits,
                    Some(cross_id),
                )?;
                let Some(slot) = usize::try_from(cross_id.ordinal())
                    .ok()
                    .and_then(|index| next_bin_hits.get_mut(index))
                else {
                    return Err(CrossSampleError::UnknownBin {
                        cross: self.name.clone(),
                        axis: CrossAxis::Left,
                        coverpoint: self.left.name().to_owned(),
                        bin: *left_id,
                    });
                };
                *slot = Some(next);
                matched_bins.push(cross_id);
            }
        }

        Ok(PreparedCrossSample {
            next_samples,
            next_bin_hits: next_bin_hits.into_boxed_slice(),
            outcome: PreparedCrossOutcome::Hit {
                bins: matched_bins.into_boxed_slice(),
                left_disposition: left.disposition(),
                right_disposition: right.disposition(),
            },
        })
    }

    /// Validates source provenance for one axis.
    fn validate_source(
        &self,
        sample: &CoverpointSample,
        axis: &CrossAxisDefinition,
        side: CrossAxis,
    ) -> Result<(), CrossSampleError> {
        if sample.originates_from(&axis.source) {
            return Ok(());
        }

        Err(CrossSampleError::SourceMismatch {
            cross: self.name.clone(),
            axis: side,
            expected: axis.name().to_owned(),
            actual: sample.coverpoint_name().to_owned(),
        })
    }

    /// Validates all source-bin IDs for one axis.
    fn validate_bins(
        &self,
        sample: &CoverpointSample,
        axis: &CrossAxisDefinition,
        side: CrossAxis,
    ) -> Result<(), CrossSampleError> {
        for id in sample.matched_bins() {
            if !axis.contains(*id) {
                return Err(CrossSampleError::UnknownBin {
                    cross: self.name.clone(),
                    axis: side,
                    coverpoint: axis.name().to_owned(),
                    bin: *id,
                });
            }
        }
        Ok(())
    }

    /// Commits an entirely preflighted cross sample.
    fn commit_sample(&mut self, prepared: &PreparedCrossSample) {
        self.samples = prepared.next_samples;
        if let PreparedCrossOutcome::Skipped {
            next_skipped_samples,
            ..
        } = prepared.outcome
        {
            self.skipped_samples = next_skipped_samples;
        }
        for (bin, next_hits) in self.bins.iter_mut().zip(prepared.next_bin_hits.iter()) {
            if let Some(next_hits) = *next_hits {
                bin.hits = next_hits;
            }
        }
    }
}

/// Fully preflighted cross state transition.
struct PreparedCrossSample {
    /// Complete post-sample total count.
    next_samples: u64,
    /// Complete post-sample per-bin hit counts.
    next_bin_hits: Box<[Option<u64>]>,
    /// Prepared public outcome.
    outcome: PreparedCrossOutcome,
}

/// Prepared semantic result.
enum PreparedCrossOutcome {
    /// Matching cross bins.
    Hit {
        /// IDs in row-major order.
        bins: Box<[CrossBinId]>,
        /// Left disposition.
        left_disposition: CoverageSampleDisposition,
        /// Right disposition.
        right_disposition: CoverageSampleDisposition,
    },
    /// Skipped cross sample.
    Skipped {
        /// Complete post-sample skipped count.
        next_skipped_samples: u64,
        /// Left disposition.
        left_disposition: CoverageSampleDisposition,
        /// Right disposition.
        right_disposition: CoverageSampleDisposition,
    },
}

impl PreparedCrossOutcome {
    /// Converts the prepared result to its public representation.
    fn into_sample(self) -> CrossSample {
        match self {
            Self::Hit {
                bins,
                left_disposition,
                right_disposition,
            } => CrossSample {
                disposition: CrossSampleDisposition::Hit,
                left_disposition,
                right_disposition,
                matched_bins: bins,
            },
            Self::Skipped {
                left_disposition,
                right_disposition,
                ..
            } => CrossSample {
                disposition: CrossSampleDisposition::Skipped,
                left_disposition,
                right_disposition,
                matched_bins: Box::default(),
            },
        }
    }
}

/// Validates cardinality before allocating generated bins.
fn cross_cardinality(
    name: &str,
    left_bins: usize,
    right_bins: usize,
    limit: usize,
) -> Result<usize, CrossBuildError> {
    let total_bins =
        left_bins
            .checked_mul(right_bins)
            .ok_or_else(|| CrossBuildError::CardinalityOverflow {
                cross: name.to_owned(),
                left_bins,
                right_bins,
            })?;
    if total_bins > limit {
        return Err(CrossBuildError::BinLimitExceeded {
            cross: name.to_owned(),
            left_bins,
            right_bins,
            total_bins,
            limit,
        });
    }
    let maximum_bin_count =
        usize::try_from(u32::MAX).map_or(usize::MAX, |maximum| maximum.saturating_add(1));

    if total_bins > maximum_bin_count {
        return Err(CrossBuildError::TooManyBins {
            cross: name.to_owned(),
            total_bins,
        });
    }
    Ok(total_bins)
}

/// Validates and constructs one complete two-way cross.
fn build_cross(builder: Cross2Builder) -> Result<Cross2, CrossBuildError> {
    let Cross2Builder {
        name,
        left,
        right,
        required_hits,
        max_bins,
    } = builder;
    if !is_valid_coverage_identifier(&name) {
        return Err(CrossBuildError::InvalidName { name });
    }
    let Some(required_hits) = NonZeroU64::new(required_hits) else {
        return Err(CrossBuildError::ZeroRequiredHits { cross: name });
    };
    if max_bins == 0 {
        return Err(CrossBuildError::ZeroBinLimit { cross: name });
    }
    let total_bins = cross_cardinality(&name, left.bins.len(), right.bins.len(), max_bins)?;
    let mut bins = Vec::with_capacity(total_bins);
    let mut bin_lookup = BTreeMap::new();
    for left_bin in &left.bins {
        for right_bin in &right.bins {
            let ordinal =
                u32::try_from(bins.len()).map_err(|_error| CrossBuildError::TooManyBins {
                    cross: name.clone(),
                    total_bins,
                })?;
            let id = CrossBinId::new(ordinal);
            bin_lookup.insert((left_bin.id, right_bin.id), id);
            bins.push(CrossBin {
                id,
                left_bin_id: left_bin.id,
                left_bin_name: Arc::clone(&left_bin.name),
                right_bin_id: right_bin.id,
                right_bin_name: Arc::clone(&right_bin.name),
                required_hits,
                hits: 0,
            });
        }
    }
    Ok(Cross2 {
        name,
        left,
        right,
        bins,
        bin_lookup,
        samples: 0,
        skipped_samples: 0,
    })
}

/// Checks one cross counter increment.
fn checked_increment(
    cross: &str,
    current: u64,
    counter: CrossCounterKind,
    bin: Option<CrossBinId>,
) -> Result<u64, CrossSampleError> {
    current
        .checked_add(1)
        .ok_or_else(|| CrossSampleError::CounterOverflow {
            cross: cross.to_owned(),
            bin,
            counter,
        })
}

#[cfg(test)]
mod tests {
    use crate::{Bin, BinId, Coverpoint, Cross2, CrossAxis, CrossBinId, CrossSampleError};

    #[test]
    fn builds_row_major_complete_cross() {
        let left = coverpoint("left", [1, 2]);
        let right = coverpoint("right", [3, 4]);
        let cross = Cross2::builder("left_x_right", &left, &right)
            .build()
            .expect("valid cross");

        let pairs = cross
            .bins()
            .iter()
            .map(|bin| (bin.left_bin_id(), bin.right_bin_id()))
            .collect::<Vec<_>>();

        assert_eq!(
            pairs,
            [
                (BinId::new(0), BinId::new(0)),
                (BinId::new(0), BinId::new(1)),
                (BinId::new(1), BinId::new(0)),
                (BinId::new(1), BinId::new(1)),
            ]
        );
    }

    #[test]
    fn samples_cartesian_product_atomically() {
        let mut left = Coverpoint::builder("left")
            .bin(Bin::inclusive_range("all", 0_u8, 2))
            .bin(Bin::value("one", 1))
            .build()
            .expect("valid coverpoint");
        let mut right = Coverpoint::builder("right")
            .bin(Bin::inclusive_range("all", 0_u8, 2))
            .bin(Bin::value("one", 1))
            .build()
            .expect("valid coverpoint");
        let mut cross = Cross2::builder("left_x_right", &left, &right)
            .build()
            .expect("valid cross");
        let left_sample = left.sample(&1).expect("normal sample");
        let right_sample = right.sample(&1).expect("normal sample");
        let sample = cross
            .sample(&left_sample, &right_sample)
            .expect("cross sample");

        assert_eq!(
            sample.matched_bins(),
            [
                CrossBinId::new(0),
                CrossBinId::new(1),
                CrossBinId::new(2),
                CrossBinId::new(3)
            ]
        );
        assert!(cross.bins().iter().all(|bin| bin.hits() == 1));
        assert_eq!(cross.sample_count(), 1);
    }

    #[test]
    fn ignored_axis_skips_cross() {
        let mut left = Coverpoint::builder("left")
            .bin(Bin::value("one", 1_u8))
            .ignore_bin(Bin::value("zero", 0))
            .build()
            .expect("valid coverpoint");
        let mut right = coverpoint("right", [1]);
        let mut cross = Cross2::builder("left_x_right", &left, &right)
            .build()
            .expect("valid cross");
        let left_sample = left.sample(&0).expect("ignored sample");
        let right_sample = right.sample(&1).expect("normal sample");
        let sample = cross
            .sample(&left_sample, &right_sample)
            .expect("skipped cross sample");

        assert!(sample.skipped());
        assert_eq!(cross.sample_count(), 1);
        assert_eq!(cross.skipped_sample_count(), 1);
        assert!(cross.bins().iter().all(|bin| bin.hits() == 0));
    }

    #[test]
    fn rejects_same_named_different_source() {
        let left = coverpoint("same", [1]);
        let mut other_left = coverpoint("same", [1]);
        let mut right = coverpoint("right", [1]);
        let mut cross = Cross2::builder("same_x_right", &left, &right)
            .build()
            .expect("valid cross");
        let left_sample = other_left.sample(&1).expect("normal sample");
        let right_sample = right.sample(&1).expect("normal sample");
        let error = cross
            .sample(&left_sample, &right_sample)
            .expect_err("source mismatch");

        assert_eq!(error.axis(), Some(CrossAxis::Left));
        assert_eq!(cross.sample_count(), 0);
        assert!(matches!(error, CrossSampleError::SourceMismatch { .. }));
    }

    fn coverpoint(name: &str, values: impl IntoIterator<Item = u8>) -> Coverpoint<u8> {
        values
            .into_iter()
            .fold(Coverpoint::builder(name), |builder, value| {
                builder.bin(Bin::value(format!("bin_{value}"), value))
            })
            .build()
            .expect("valid coverpoint")
    }
}
