use std::collections::BTreeSet;
use std::num::NonZeroU64;

use crate::coverage::{
    Bin, BinId, BinKind, CoverageBuildError, CoverageRatio, CoverageSampleError, CoverpointBin,
    MatcherValidationError,
};

/// Bin defined waiting to be validated and bound.
struct PendingBin<T> {
    /// Requested semantic role.
    kind: BinKind,

    /// User-provided definition.
    definition: Bin<T>,
}

/// Consuming fluent builder for one typed functional coverpoint.
///
/// Build validates identifiers, uniqueness across every bin kind, matcher
/// definitions, and thresholds. At least one normal bin is required. Overlap
/// between matchers is allowed and is resolved at sampling by category
/// precedence.
pub struct CoverpointBuilder<T> {
    /// Stable coverpoint name.
    name: String,

    /// Pending bins in declaration order.
    bins: Vec<PendingBin<T>>,
}

impl<T> CoverpointBuilder<T> {
    /// Adds a normal bin.
    #[must_use]
    pub fn bin(mut self, definition: Bin<T>) -> Self {
        self.bins.push(PendingBin {
            kind: BinKind::Normal,
            definition,
        });
        self
    }

    /// Adds an ignore bin.
    #[must_use]
    pub fn ignore_bin(mut self, definition: Bin<T>) -> Self {
        self.bins.push(PendingBin {
            kind: BinKind::Ignore,
            definition,
        });
        self
    }

    /// Adds an illegal bin.
    #[must_use]
    pub fn illegal_bin(mut self, definition: Bin<T>) -> Self {
        self.bins.push(PendingBin {
            kind: BinKind::Illegal,
            definition,
        });

        self
    }

    /// Validates and builds the coverpoint.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageBuildError`] if:
    /// - coverpoint or bin naming conventions are violated,
    /// - no normal bins were configured for the coverpoint,
    /// - a bin with invalid values was configured for coverpoint,
    /// - invalid hit threshold was configured for a coverpoint bin.
    pub fn build(self) -> Result<Coverpoint<T>, CoverageBuildError> {
        build_coverpoint(self)
    }
}

/// Explicitly owned typed functional coverpoint and its runtime counters.
///
/// Sampling occurs only through [`Self::sample`]. A coverpoint has no global
/// registration and is not automatically tied to `Sample`, `Scoreboard`, a
/// DUT, `Testbench`, or `vvm::test`. Only normal bins contribute to completion.
pub struct Coverpoint<T> {
    /// Stable coverpoint name.
    name: String,

    /// Validated bins in declaration order.
    bins: Vec<CoverpointBin<T>>,

    /// Total attempted samples.
    samples: u64,

    /// Samples excluded by ignore bins.
    ignored_samples: u64,

    /// Samples rejected by illegal bins.
    illegal_samples: u64,

    /// Samples matching no bin.
    unmatched_samples: u64,
}

impl<T> Coverpoint<T> {
    /// Starts constructing a coverpoint.
    #[must_use]
    pub fn builder(name: impl Into<String>) -> CoverpointBuilder<T> {
        CoverpointBuilder {
            name: name.into(),
            bins: Vec::new(),
        }
    }

    /// Returns the coverpoint name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns coverpoint bins in declaration order.
    #[must_use]
    pub fn bins(&self) -> &[CoverpointBin<T>] {
        &self.bins
    }

    /// Returns normal bins in declaration order.
    pub fn normal_bins(&self) -> impl Iterator<Item = &CoverpointBin<T>> {
        self.bins.iter().filter(|bin| bin.kind() == BinKind::Normal)
    }

    /// Returns ignored bins in declaration order.
    pub fn ignore_bins(&self) -> impl Iterator<Item = &CoverpointBin<T>> {
        self.bins.iter().filter(|bin| bin.kind() == BinKind::Ignore)
    }

    /// Returns illegal bins in declaration order.
    pub fn illegal_bins(&self) -> impl Iterator<Item = &CoverpointBin<T>> {
        self.bins
            .iter()
            .filter(|bin| bin.kind() == BinKind::Illegal)
    }

    /// Finds one bin by its coverpoint-local identifier.
    #[must_use]
    pub fn bin(&self, id: BinId) -> Option<&CoverpointBin<T>> {
        let index = usize::try_from(id.ordinal()).ok()?;

        self.bins.get(index)
    }

    /// Returns the number of attempted samples.
    ///
    /// This includes normal, ignored, illegal and unmatched samples.
    #[must_use]
    pub const fn sample_count(&self) -> u64 {
        self.samples
    }

    /// Returns the number of samples excluded by ignore bins.
    #[must_use]
    pub const fn ignored_sample_count(&self) -> u64 {
        self.ignored_samples
    }

    /// Returns the number of samples rejected by illegal bins.
    #[must_use]
    pub const fn illegal_sample_count(&self) -> u64 {
        self.illegal_samples
    }

    /// Returns the number of samples matching no declared bin.
    #[must_use]
    pub const fn unmatched_sample_count(&self) -> u64 {
        self.unmatched_samples
    }

    /// Samples one value into this coverpoint.
    ///
    /// Matching precedence is:
    ///
    /// 1. illegal bins;
    /// 2. ignore bins;
    /// 3. normal bins;
    /// 4. unmatched.
    ///
    /// All matching bins within the selected category increment. Bins
    /// from lower-precedence categories do not increment.
    ///
    /// Counter updates are atomic with respect to counter-overflow
    /// failures. Illegal-bin counters are committed before an
    /// [`CoverageSampleError::IllegalBinHit`] error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`CoverageSampleError::IllegalBinHit`] when one or more
    /// illegal bins match.
    ///
    /// Returns [`CoverageSampleError::CounterOverflow`] when any affected
    /// coverage counter cannot be incremented.
    pub fn sample(&mut self, sampled: &T) -> Result<CoverpointSample, CoverageSampleError>
    where
        T: std::fmt::Debug,
    {
        let prepared = self.prepare_sample(sampled)?;

        self.commit_sample(&prepared);

        match prepared.outcome {
            PreparedOutcome::Hit { bins } => Ok(CoverpointSample::new_hit(bins)),

            PreparedOutcome::Ignored { .. } => Ok(CoverpointSample::new_ignored()),

            PreparedOutcome::Unmatched { .. } => Ok(CoverpointSample::new_unmatched()),

            PreparedOutcome::Illegal {
                bins,
                sampled_value,
                ..
            } => Err(CoverageSampleError::IllegalBinHit {
                coverpoint: self.name.clone(),
                bins,
                sampled_value,
            }),
        }
    }

    /// Returns the exact normal-bin coverage ratio.
    #[must_use]
    pub fn coverage(&self) -> CoverageRatio {
        let covered = self.normal_bins().filter(|bin| bin.covered()).count();

        let uncovered = self.normal_bins().filter(|bin| !bin.covered()).count();

        let total = self.normal_bins().count();

        CoverageRatio::new(covered, uncovered, total)
    }

    /// Returns uncovered normal bins in declaration order.
    pub fn uncovered_bins(&self) -> impl Iterator<Item = &CoverpointBin<T>> {
        self.normal_bins().filter(|bin| !bin.covered())
    }

    /// Returns covered normal bins in declaration order.
    pub fn covered_bins(&self) -> impl Iterator<Item = &CoverpointBin<T>> {
        self.normal_bins().filter(|bin| bin.covered())
    }

    /// Prepare full sample.
    fn prepare_sample(&self, sampled: &T) -> Result<PreparedSample, CoverageSampleError>
    where
        T: std::fmt::Debug,
    {
        let matches = match_bins(&self.bins, sampled);

        let category = selected_category(&self.bins, &matches);

        let next_samples =
            checked_increment(&self.name, self.samples, CoverageCounterKind::Samples, None)?;

        let selected_kind = match category {
            SampleCategory::Illegal => Some(BinKind::Illegal),

            SampleCategory::Ignored => Some(BinKind::Ignore),

            SampleCategory::Normal => Some(BinKind::Normal),

            SampleCategory::Unmatched => None,
        };

        let next_bin_hits = prepared_bin_hits(&self.name, &self.bins, &matches, selected_kind)?;

        let outcome = match category {
            SampleCategory::Normal => {
                let bins = self
                    .bins
                    .iter()
                    .zip(&matches)
                    .filter_map(|(bin, matched)| {
                        (*matched && bin.kind() == BinKind::Normal).then_some(bin.id())
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();

                PreparedOutcome::Hit { bins }
            }

            SampleCategory::Ignored => {
                let next = checked_increment(
                    &self.name,
                    self.ignored_samples,
                    CoverageCounterKind::IgnoredSamples,
                    None,
                )?;

                PreparedOutcome::Ignored {
                    next_ignored_samples: next,
                }
            }

            SampleCategory::Unmatched => {
                let next = checked_increment(
                    &self.name,
                    self.unmatched_samples,
                    CoverageCounterKind::UnmatchedSamples,
                    None,
                )?;

                PreparedOutcome::Unmatched {
                    next_unmatched_samples: next,
                }
            }

            SampleCategory::Illegal => {
                let next = checked_increment(
                    &self.name,
                    self.illegal_samples,
                    CoverageCounterKind::IllegalSamples,
                    None,
                )?;

                let bins = self
                    .bins
                    .iter()
                    .zip(&matches)
                    .filter(|&(bin, matched)| *matched && bin.kind() == BinKind::Illegal)
                    .map(|(bin, _matched)| bin.name().to_owned())
                    .collect::<Vec<_>>()
                    .into_boxed_slice();

                PreparedOutcome::Illegal {
                    next_illegal_samples: next,
                    bins,
                    sampled_value: format!("{sampled:?}"),
                }
            }
        };

        Ok(PreparedSample {
            next_samples,
            next_bin_hits,
            outcome,
        })
    }

    /// Commit prepared sample.
    fn commit_sample(&mut self, prepared: &PreparedSample) {
        self.samples = prepared.next_samples;

        match prepared.outcome {
            PreparedOutcome::Hit { .. } => {}

            PreparedOutcome::Ignored {
                next_ignored_samples,
            } => {
                self.ignored_samples = next_ignored_samples;
            }

            PreparedOutcome::Unmatched {
                next_unmatched_samples,
            } => {
                self.unmatched_samples = next_unmatched_samples;
            }

            PreparedOutcome::Illegal {
                next_illegal_samples,
                ..
            } => {
                self.illegal_samples = next_illegal_samples;
            }
        }

        for (bin, next_hits) in self.bins.iter_mut().zip(prepared.next_bin_hits.iter()) {
            if let Some(next_hits) = *next_hits {
                bin.set_hits(next_hits);
            }
        }
    }
}

/// Kind of functional coverage counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageCounterKind {
    /// Total attempted samples.
    Samples,

    /// Ignored samples.
    IgnoredSamples,

    /// Illegal samples.
    IllegalSamples,

    /// Unmatched samples.
    UnmatchedSamples,

    /// One bin's hit count.
    BinHits,
}

impl std::fmt::Display for CoverageCounterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Samples => f.write_str("total sample count"),
            Self::IgnoredSamples => f.write_str("ignored sample count"),
            Self::IllegalSamples => f.write_str("illegal sample count"),
            Self::UnmatchedSamples => f.write_str("unmatched sample count"),
            Self::BinHits => f.write_str("bin hit count"),
        }
    }
}

/// Disposition of one successfully processed non-illegal sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageSampleDisposition {
    /// One or more normal bins were hit.
    Hit,

    /// An ignore bin excluded the sample.
    Ignored,

    /// No bin matched.
    Unmatched,
}

/// Result of one successful non-illegal coverpoint sample.
///
/// [`Self::matched_bins`] contains normal-bin IDs only, in declaration order.
/// Ignored and unmatched samples contain no IDs; illegal samples instead return
/// [`CoverageSampleError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverpointSample {
    /// Selected disposition.
    disposition: CoverageSampleDisposition,

    /// Matching normal bins.
    matched_bins: Box<[BinId]>,
}

impl CoverpointSample {
    /// Creates a normal-hit sample result.
    const fn new_hit(matched_bins: Box<[BinId]>) -> Self {
        Self {
            disposition: CoverageSampleDisposition::Hit,
            matched_bins,
        }
    }

    /// Creates an ignored sample result.
    fn new_ignored() -> Self {
        Self {
            disposition: CoverageSampleDisposition::Ignored,
            matched_bins: Box::default(),
        }
    }

    /// Creates an unmatched sample result.
    fn new_unmatched() -> Self {
        Self {
            disposition: CoverageSampleDisposition::Unmatched,
            matched_bins: Box::default(),
        }
    }

    /// Returns the sample disposition.
    #[must_use]
    pub const fn disposition(&self) -> CoverageSampleDisposition {
        self.disposition
    }

    /// Returns matching normal-bin identifiers.
    #[must_use]
    pub fn matched_bins(&self) -> &[BinId] {
        &self.matched_bins
    }

    /// Returns whether one or more normal bins were hit.
    #[must_use]
    pub const fn hit(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Hit,)
    }

    /// Returns whether an ignore bin excluded the sample.
    #[must_use]
    pub const fn ignored(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Ignored,)
    }

    /// Returns whether no declared bin matched.
    #[must_use]
    pub const fn unmatched(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Unmatched,)
    }
}

/// Highest-precedence category selected for one sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SampleCategory {
    /// Illegal bin matched.
    Illegal,

    /// Ignore bin matched without an illegal match.
    Ignored,

    /// Normal bin matched without an excluded match.
    Normal,

    /// No bin matched.
    Unmatched,
}

/// Prepared semantic result.
enum PreparedOutcome {
    /// Matching normal bins.
    Hit {
        /// IDs in declaration order.
        bins: Box<[BinId]>,
    },

    /// Ignored sample.
    Ignored {
        /// Complete post-sample ignored count.
        next_ignored_samples: u64,
    },

    /// Unmatched sample.
    Unmatched {
        /// Complete post-sample unmatched count.
        next_unmatched_samples: u64,
    },

    /// Illegal sample.
    Illegal {
        /// Complete post-sample illegal count.
        next_illegal_samples: u64,

        /// Matching illegal names.
        bins: Box<[String]>,

        /// Debug representation of the sample.
        sampled_value: String,
    },
}

/// Fully preflighted state transition.
struct PreparedSample {
    /// Complete post-sample total count.
    next_samples: u64,

    /// Prepared per-bin hit counters.
    ///
    /// The slice has exactly one entry per live bin.
    next_bin_hits: Box<[Option<u64>]>,

    /// Semantic result.
    outcome: PreparedOutcome,
}

/// Returns whether a name is a valid coverage identifier.
fn is_valid_coverage_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();

    let Some(first) = bytes.next() else {
        return false;
    };

    let valid_first = first.is_ascii_alphabetic() || first == b'_';

    valid_first && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Validates and normalizes one hit threshold.
fn validated_required_hits(
    coverpoint: &str,
    kind: BinKind,
    definition: &Bin<impl Sized>,
) -> Result<NonZeroU64, CoverageBuildError> {
    let requested = definition.requested_hits();

    match kind {
        BinKind::Normal => {
            NonZeroU64::new(requested).ok_or_else(|| CoverageBuildError::ZeroRequiredHits {
                coverpoint: coverpoint.to_owned(),
                bin: definition.name().to_owned(),
            })
        }

        BinKind::Ignore | BinKind::Illegal => {
            if requested != 1 {
                return Err(CoverageBuildError::HitRequirementOnExcludedBin {
                    coverpoint: coverpoint.to_owned(),
                    bin: definition.name().to_owned(),
                    kind,
                });
            }

            Ok(NonZeroU64::MIN)
        }
    }
}

/// Validates bin value-matcher.
fn validate_matcher<T>(coverpoint: &str, definition: &Bin<T>) -> Result<(), CoverageBuildError> {
    match definition.matcher().validate() {
        Ok(()) => Ok(()),

        Err(MatcherValidationError::EmptyValueSet) => Err(CoverageBuildError::EmptyValueSet {
            coverpoint: coverpoint.to_owned(),
            bin: definition.name().to_owned(),
        }),

        Err(MatcherValidationError::DuplicateValue { first, duplicate }) => {
            Err(CoverageBuildError::DuplicateValue {
                coverpoint: coverpoint.to_owned(),
                bin: definition.name().to_owned(),
                first,
                duplicate,
            })
        }

        Err(MatcherValidationError::InvalidInclusiveRange) => {
            Err(CoverageBuildError::InvalidInclusiveRange {
                coverpoint: coverpoint.to_owned(),
                bin: definition.name().to_owned(),
            })
        }
    }
}

/// Builds one coverage coverpoint.
fn build_coverpoint<T>(builder: CoverpointBuilder<T>) -> Result<Coverpoint<T>, CoverageBuildError> {
    let CoverpointBuilder { name, bins } = builder;

    if !is_valid_coverage_identifier(&name) {
        return Err(CoverageBuildError::InvalidCoverpointName { name });
    }

    let mut names = BTreeSet::new();

    let mut has_normal_bin = false;

    let mut live_bins = Vec::with_capacity(bins.len());

    for (ordinal, pending) in bins.into_iter().enumerate() {
        let PendingBin { kind, definition } = pending;

        if !is_valid_coverage_identifier(definition.name()) {
            return Err(CoverageBuildError::InvalidBinName {
                coverpoint: name,
                bin: definition.name().to_owned(),
            });
        }

        if !names.insert(definition.name().to_owned()) {
            return Err(CoverageBuildError::DuplicateBinName {
                coverpoint: name,
                bin: definition.name().to_owned(),
            });
        }

        validate_matcher(&name, &definition)?;

        let required_hits = validated_required_hits(&name, kind, &definition)?;

        let ordinal = u32::try_from(ordinal).map_err(|_error| CoverageBuildError::TooManyBins {
            coverpoint: name.clone(),
        })?;

        if kind == BinKind::Normal {
            has_normal_bin = true;
        }

        live_bins.push(CoverpointBin::new(
            BinId::new(ordinal),
            kind,
            definition,
            required_hits,
        ));
    }

    if !has_normal_bin {
        return Err(CoverageBuildError::NoNormalBins { coverpoint: name });
    }

    Ok(Coverpoint {
        name,
        bins: live_bins,
        samples: 0,
        ignored_samples: 0,
        illegal_samples: 0,
        unmatched_samples: 0,
    })
}

/// Checked incremet helper.
fn checked_increment(
    coverpoint: &str,
    current: u64,
    counter: CoverageCounterKind,
    bin: Option<&str>,
) -> Result<u64, CoverageSampleError> {
    current
        .checked_add(1)
        .ok_or_else(|| CoverageSampleError::CounterOverflow {
            coverpoint: coverpoint.to_owned(),
            bin: bin.map(str::to_owned),
            counter,
        })
}

/// Matches all bins.
fn match_bins<T>(bins: &[CoverpointBin<T>], sampled: &T) -> Box<[bool]> {
    bins.iter()
        .map(|bin| bin.matches(sampled))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

/// Selects bin precedence category.
fn selected_category<T>(bins: &[CoverpointBin<T>], matches: &[bool]) -> SampleCategory {
    let has_kind = |kind: BinKind| {
        bins.iter()
            .zip(matches)
            .any(|(bin, matched)| *matched && bin.kind() == kind)
    };

    if has_kind(BinKind::Illegal) {
        SampleCategory::Illegal
    } else if has_kind(BinKind::Ignore) {
        SampleCategory::Ignored
    } else if has_kind(BinKind::Normal) {
        SampleCategory::Normal
    } else {
        SampleCategory::Unmatched
    }
}

/// Prepare bin hits.
fn prepared_bin_hits<T>(
    coverpoint: &str,
    bins: &[CoverpointBin<T>],
    matches: &[bool],
    selected_kind: Option<BinKind>,
) -> Result<Box<[Option<u64>]>, CoverageSampleError> {
    bins.iter()
        .zip(matches)
        .map(|(bin, matched)| {
            let selected = *matched && selected_kind == Some(bin.kind());

            if !selected {
                return Ok(None);
            }

            let next =
                bin.checked_next_hits()
                    .ok_or_else(|| CoverageSampleError::CounterOverflow {
                        coverpoint: coverpoint.to_owned(),
                        bin: Some(bin.name().to_owned()),
                        counter: CoverageCounterKind::BinHits,
                    })?;

            Ok(Some(next))
        })
        .collect::<Result<Vec<_>, CoverageSampleError>>()
        .map(Vec::into_boxed_slice)
}

#[cfg(test)]
mod tests {
    use crate::{
        Bin, BinId, BinKind, CoverageBuildError, CoverageCounterKind, CoverageSampleDisposition,
        CoverageSampleError, Coverpoint, CoverpointBin,
    };

    /// Complete mutable counter state used by atomicity tests.
    #[derive(Debug, PartialEq, Eq)]
    struct CounterSnapshot {
        /// Total attempted samples.
        samples: u64,
        /// Ignored sample count.
        ignored_samples: u64,
        /// Illegal sample count.
        illegal_samples: u64,
        /// Unmatched sample count.
        unmatched_samples: u64,
        /// Bin hit counters in declaration order.
        bin_hits: Vec<u64>,
    }

    #[test]
    fn builds_coverpoint() {
        let _coverage = Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8))
            .build()
            .expect("valid coverpoint");
    }

    #[test]
    fn preserves_coverpoint_name() {
        assert_eq!(coverage().name(), "opcode");
    }

    #[test]
    fn preserves_mixed_bin_declaration_order() {
        let coverage = mixed_coverage();
        let names = coverage
            .bins()
            .iter()
            .map(CoverpointBin::name)
            .collect::<Vec<_>>();

        assert_eq!(names, ["normal_a", "ignore_a", "illegal_a", "normal_b"]);
    }

    #[test]
    fn assigns_bin_ids_in_global_declaration_order() {
        let ids = mixed_coverage()
            .bins()
            .iter()
            .map(CoverpointBin::id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            [BinId::new(0), BinId::new(1), BinId::new(2), BinId::new(3)]
        );
    }

    #[test]
    fn all_live_bins_begin_with_zero_hits() {
        assert!(coverage().bins().iter().all(|bin| bin.hits() == 0));
    }

    #[test]
    fn all_sample_counters_begin_at_zero() {
        let coverage = coverage();

        assert_eq!(coverage.sample_count(), 0);
        assert_eq!(coverage.ignored_sample_count(), 0);
        assert_eq!(coverage.illegal_sample_count(), 0);
        assert_eq!(coverage.unmatched_sample_count(), 0);
    }

    #[test]
    fn accepts_valid_coverage_identifiers() {
        for name in [
            "opcode",
            "response_code",
            "_reserved",
            "length_0_to_15",
            "A",
            "a9",
        ] {
            let _coverage = Coverpoint::builder(name)
                .bin(Bin::value("bin", 1_u8))
                .build()
                .expect("valid coverpoint identifier");
        }
    }

    #[test]
    fn rejects_empty_coverpoint_name() {
        invalid_coverpoint_name("");
    }

    #[test]
    fn rejects_coverpoint_name_starting_with_digit() {
        invalid_coverpoint_name("1opcode");
    }

    #[test]
    fn rejects_coverpoint_name_containing_dot() {
        invalid_coverpoint_name("opcode.value");
    }

    #[test]
    fn rejects_coverpoint_name_containing_space() {
        invalid_coverpoint_name("opcode value");
    }

    #[test]
    fn rejects_control_character_coverpoint_name() {
        invalid_coverpoint_name("opcode_");
    }

    #[test]
    fn rejects_unicode_coverpoint_name_when_not_ascii_identifier() {
        invalid_coverpoint_name("opcode_\u{00e9}");
    }

    #[test]
    fn rejects_empty_bin_name() {
        invalid_bin_name("");
    }

    #[test]
    fn rejects_bin_name_starting_with_digit() {
        invalid_bin_name("1bin");
    }

    #[test]
    fn rejects_bin_name_containing_dot() {
        invalid_bin_name("my.bin");
    }

    #[test]
    fn rejects_bin_name_containing_space() {
        invalid_bin_name("my bin");
    }

    #[test]
    fn rejects_duplicate_normal_bin_name() {
        duplicate_name_error(
            Coverpoint::builder("opcode")
                .bin(Bin::value("same", 1_u8))
                .bin(Bin::value("same", 2)),
        );
    }

    #[test]
    fn rejects_duplicate_name_between_normal_and_ignore() {
        duplicate_name_error(
            Coverpoint::builder("opcode")
                .bin(Bin::value("same", 1_u8))
                .ignore_bin(Bin::value("same", 2)),
        );
    }

    #[test]
    fn rejects_duplicate_name_between_normal_and_illegal() {
        duplicate_name_error(
            Coverpoint::builder("opcode")
                .bin(Bin::value("same", 1_u8))
                .illegal_bin(Bin::value("same", 2)),
        );
    }

    #[test]
    fn rejects_duplicate_name_between_ignore_and_illegal() {
        duplicate_name_error(
            Coverpoint::builder("opcode")
                .bin(Bin::value("normal", 1_u8))
                .ignore_bin(Bin::value("same", 2))
                .illegal_bin(Bin::value("same", 3)),
        );
    }

    #[test]
    fn rejects_coverpoint_without_normal_bins() {
        let error = Coverpoint::builder("opcode")
            .ignore_bin(Bin::value("reset", 0_u8))
            .illegal_bin(Bin::value("reserved", u8::MAX))
            .build()
            .err()
            .expect("coverpoint must contain a normal bin");

        assert_eq!(
            error,
            CoverageBuildError::NoNormalBins {
                coverpoint: "opcode".into()
            }
        );
    }

    #[test]
    fn rejects_empty_value_set() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::values("values", Vec::<u8>::new()))
            .build()
            .err()
            .expect("empty value set must fail");

        assert_eq!(
            error,
            CoverageBuildError::EmptyValueSet {
                coverpoint: "opcode".into(),
                bin: "values".into()
            }
        );
    }

    #[test]
    fn rejects_duplicate_values_in_set() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::values("values", [1_u8, 2, 1]))
            .build()
            .err()
            .expect("duplicate values must fail");

        assert_eq!(
            error,
            CoverageBuildError::DuplicateValue {
                coverpoint: "opcode".into(),
                bin: "values".into(),
                first: 0,
                duplicate: 2
            }
        );
    }

    #[test]
    fn rejects_reversed_inclusive_range() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::inclusive_range("range", 2_u8, 1))
            .build()
            .err()
            .expect("reversed range must fail");

        assert_eq!(
            error,
            CoverageBuildError::InvalidInclusiveRange {
                coverpoint: "opcode".into(),
                bin: "range".into()
            }
        );
    }

    #[test]
    fn rejects_incomparable_inclusive_range() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::inclusive_range("range", f32::NAN, 1.0))
            .build()
            .err()
            .expect("incomparable range must fail");

        assert_eq!(
            error,
            CoverageBuildError::InvalidInclusiveRange {
                coverpoint: "opcode".into(),
                bin: "range".into()
            }
        );
    }

    #[test]
    fn normal_bin_defaults_to_one_required_hit() {
        assert_eq!(
            coverage().bins().first().map(CoverpointBin::required_hits),
            Some(1)
        );
    }

    #[test]
    fn normal_bin_accepts_custom_nonzero_threshold() {
        let coverage = Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8).at_least(2))
            .build()
            .expect("valid threshold");

        assert_eq!(
            coverage.bins().first().map(CoverpointBin::required_hits),
            Some(2)
        );
    }

    #[test]
    fn rejects_zero_normal_hit_threshold() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8).at_least(0))
            .build()
            .err()
            .expect("zero threshold must fail");

        assert_eq!(
            error,
            CoverageBuildError::ZeroRequiredHits {
                coverpoint: "opcode".into(),
                bin: "read".into()
            }
        );
    }

    #[test]
    fn rejects_custom_ignore_hit_threshold() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::value("normal", 1_u8))
            .ignore_bin(Bin::value("ignored", 0).at_least(2))
            .build()
            .err()
            .expect("ignore thresholds are not supported");

        assert_eq!(
            error,
            CoverageBuildError::HitRequirementOnExcludedBin {
                coverpoint: "opcode".into(),
                bin: "ignored".into(),
                kind: BinKind::Ignore
            }
        );
    }

    #[test]
    fn rejects_custom_illegal_hit_threshold() {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::value("normal", 1_u8))
            .illegal_bin(Bin::value("illegal", 0).at_least(2))
            .build()
            .err()
            .expect("illegal thresholds are not supported");

        assert_eq!(
            error,
            CoverageBuildError::HitRequirementOnExcludedBin {
                coverpoint: "opcode".into(),
                bin: "illegal".into(),
                kind: BinKind::Illegal
            }
        );
    }

    #[test]
    fn normal_sample_increments_matching_bin() {
        let mut coverage = coverage();

        coverage.sample(&1).expect("normal sample");

        assert_eq!(coverage.bins().first().map(CoverpointBin::hits), Some(1));
    }

    #[test]
    fn normal_sample_does_not_increment_nonmatching_bins() {
        let mut coverage = coverage();

        coverage.sample(&1).expect("normal sample");

        assert_eq!(coverage.bins().get(1).map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn normal_sample_returns_matching_bin_id() {
        let mut coverage = coverage();
        let sample = coverage.sample(&1).expect("normal sample");

        assert_eq!(sample.matched_bins(), [BinId::new(0)]);
    }

    #[test]
    fn overlapping_normal_sample_increments_all_matches() {
        let mut coverage = overlapping_coverage();

        coverage.sample(&6).expect("normal sample");

        assert_eq!(bin_hits(&coverage), [1, 1, 1]);
    }

    #[test]
    fn overlapping_normal_sample_returns_all_matching_ids() {
        let mut coverage = overlapping_coverage();
        let sample = coverage.sample(&6).expect("normal sample");

        assert_eq!(
            sample.matched_bins(),
            [BinId::new(0), BinId::new(1), BinId::new(2)]
        );
        assert_eq!(sample.disposition(), CoverageSampleDisposition::Hit);
    }

    #[test]
    fn matching_ids_preserve_declaration_order() {
        let mut coverage = overlapping_coverage();
        let sample = coverage.sample(&6).expect("normal sample");

        assert_eq!(
            sample.matched_bins(),
            [BinId::new(0), BinId::new(1), BinId::new(2)]
        );
    }

    #[test]
    fn normal_sample_increments_total_sample_count() {
        let mut coverage = coverage();

        coverage.sample(&1).expect("normal sample");

        assert_eq!(coverage.sample_count(), 1);
    }

    #[test]
    fn ignored_sample_increments_matching_ignore_bin() {
        let mut coverage = ignore_coverage();

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(coverage.bins().get(1).map(CoverpointBin::hits), Some(1));
    }

    #[test]
    fn ignored_sample_increments_all_matching_ignore_bins() {
        let mut coverage = Coverpoint::builder("value")
            .bin(Bin::inclusive_range("normal", 0_u8, 15))
            .ignore_bin(Bin::value("zero_a", 0))
            .ignore_bin(Bin::inclusive_range("zero_b", 0, 0))
            .build()
            .expect("valid coverpoint");

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(bin_hits(&coverage), [0, 1, 1]);
    }

    #[test]
    fn ignored_sample_increments_total_sample_count() {
        let mut coverage = ignore_coverage();

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(coverage.sample_count(), 1);
    }

    #[test]
    fn ignored_sample_increments_ignored_sample_count() {
        let mut coverage = ignore_coverage();

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(coverage.ignored_sample_count(), 1);
    }

    #[test]
    fn ignored_sample_suppresses_normal_bin_hits() {
        let mut coverage = ignore_coverage();

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(coverage.bins().first().map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn ignored_sample_returns_ignored_disposition() {
        let mut coverage = ignore_coverage();
        let sample = coverage.sample(&0).expect("ignored sample");

        assert!(sample.ignored());
    }

    #[test]
    fn ignored_sample_returns_no_normal_bin_ids() {
        let mut coverage = ignore_coverage();
        let sample = coverage.sample(&0).expect("ignored sample");

        assert!(sample.matched_bins().is_empty());
    }

    #[test]
    fn illegal_sample_increments_matching_illegal_bin() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.bins().get(2).map(CoverpointBin::hits), Some(1));
    }

    #[test]
    fn illegal_sample_increments_all_matching_illegal_bins() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(bin_hits(&coverage), [0, 0, 1, 1]);
    }

    #[test]
    fn illegal_sample_increments_total_sample_count() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.sample_count(), 1);
    }

    #[test]
    fn illegal_sample_increments_illegal_sample_count() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.illegal_sample_count(), 1);
    }

    #[test]
    fn illegal_sample_suppresses_ignore_bin_hits() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.bins().get(1).map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn illegal_sample_suppresses_normal_bin_hits() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.bins().first().map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn illegal_sample_returns_error() {
        let mut coverage = illegal_coverage();

        assert!(matches!(
            coverage.sample(&u8::MAX),
            Err(CoverageSampleError::IllegalBinHit { .. })
        ));
    }

    #[test]
    fn illegal_error_contains_coverpoint_name() {
        assert_eq!(illegal_error().coverpoint(), "value");
    }

    #[test]
    fn illegal_error_contains_all_matching_bin_names() {
        assert_eq!(
            illegal_error().illegal_bins(),
            Some(["illegal_a".into(), "illegal_b".into()].as_slice())
        );
    }

    #[test]
    fn illegal_error_preserves_bin_declaration_order() {
        assert_eq!(
            illegal_error().illegal_bins(),
            Some(["illegal_a".into(), "illegal_b".into()].as_slice())
        );
    }

    #[test]
    fn illegal_error_contains_sampled_value() {
        assert_eq!(illegal_error().sampled_value(), Some("255"));
    }

    #[test]
    fn unmatched_sample_increments_total_sample_count() {
        let mut coverage = coverage();

        coverage.sample(&9).expect("unmatched sample");

        assert_eq!(coverage.sample_count(), 1);
    }

    #[test]
    fn unmatched_sample_increments_unmatched_sample_count() {
        let mut coverage = coverage();

        coverage.sample(&9).expect("unmatched sample");

        assert_eq!(coverage.unmatched_sample_count(), 1);
    }

    #[test]
    fn unmatched_sample_does_not_increment_any_bin() {
        let mut coverage = coverage();

        coverage.sample(&9).expect("unmatched sample");

        assert_eq!(bin_hits(&coverage), [0, 0]);
    }

    #[test]
    fn unmatched_sample_returns_unmatched_disposition() {
        let mut coverage = coverage();
        let sample = coverage.sample(&9).expect("unmatched sample");

        assert!(sample.unmatched());
    }

    #[test]
    fn unmatched_sample_returns_no_bin_ids() {
        let mut coverage = coverage();
        let sample = coverage.sample(&9).expect("unmatched sample");

        assert!(sample.matched_bins().is_empty());
    }

    #[test]
    fn illegal_takes_precedence_over_ignore() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.illegal_sample_count(), 1);
        assert_eq!(coverage.ignored_sample_count(), 0);
    }

    #[test]
    fn illegal_takes_precedence_over_normal() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(coverage.bins().first().map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn ignore_takes_precedence_over_normal() {
        let mut coverage = ignore_coverage();

        coverage.sample(&0).expect("ignored sample");

        assert_eq!(coverage.bins().first().map(CoverpointBin::hits), Some(0));
    }

    #[test]
    fn all_matches_within_selected_category_increment() {
        let mut coverage = overlapping_coverage();

        coverage.sample(&6).expect("normal sample");

        assert_eq!(bin_hits(&coverage), [1, 1, 1]);
    }

    #[test]
    fn lower_precedence_categories_do_not_increment() {
        let mut coverage = illegal_coverage();

        let _error = coverage.sample(&u8::MAX).expect_err("illegal sample");

        assert_eq!(bin_hits(&coverage), [0, 0, 1, 1]);
    }

    #[test]
    fn bin_is_uncovered_before_required_hits() {
        let coverage = threshold_coverage();

        assert!(!coverage.bins().first().is_some_and(CoverpointBin::covered));
    }

    #[test]
    fn bin_remains_uncovered_below_required_hits() {
        let mut coverage = threshold_coverage();

        coverage.sample(&1).expect("normal sample");

        assert!(!coverage.bins().first().is_some_and(CoverpointBin::covered));
    }

    #[test]
    fn bin_becomes_covered_at_required_hits() {
        let mut coverage = threshold_coverage();

        coverage.sample(&1).expect("normal sample");
        coverage.sample(&1).expect("normal sample");

        assert!(coverage.bins().first().is_some_and(CoverpointBin::covered));
    }

    #[test]
    fn bin_remains_covered_above_required_hits() {
        let mut coverage = threshold_coverage();

        for _ in 0..3 {
            coverage.sample(&1).expect("normal sample");
        }

        assert!(coverage.bins().first().is_some_and(CoverpointBin::covered));
    }

    #[test]
    fn initial_coverage_has_zero_covered_bins() {
        assert_eq!(ratio_coverage().coverage().covered(), 0);
    }

    #[test]
    fn coverage_counts_only_normal_bins() {
        assert_eq!(ratio_coverage().coverage().total(), 2);
    }

    #[test]
    fn coverage_excludes_ignore_bins_from_denominator() {
        assert_eq!(ratio_coverage().coverage().total(), 2);
    }

    #[test]
    fn coverage_excludes_illegal_bins_from_denominator() {
        assert_eq!(ratio_coverage().coverage().total(), 2);
    }

    #[test]
    fn coverage_reports_covered_count() {
        let mut coverage = ratio_coverage();

        coverage.sample(&2).expect("normal sample");

        assert_eq!(coverage.coverage().covered(), 1);
    }

    #[test]
    fn coverage_reports_uncovered_count() {
        assert_eq!(ratio_coverage().coverage().uncovered(), 2);
    }

    #[test]
    fn coverage_reports_total_normal_bins() {
        assert_eq!(ratio_coverage().coverage().total(), 2);
    }

    #[test]
    fn coverage_is_incomplete_when_any_normal_bin_is_uncovered() {
        let mut coverage = ratio_coverage();

        coverage.sample(&2).expect("normal sample");

        assert!(!coverage.coverage().is_complete());
    }

    #[test]
    fn coverage_is_complete_when_all_normal_bins_are_covered() {
        let mut coverage = ratio_coverage();

        coverage.sample(&1).expect("normal sample");
        coverage.sample(&1).expect("normal sample");
        coverage.sample(&2).expect("normal sample");

        assert!(coverage.coverage().is_complete());
    }

    #[test]
    fn uncovered_bins_returns_only_normal_bins() {
        assert!(
            ratio_coverage()
                .uncovered_bins()
                .all(|bin| bin.kind() == BinKind::Normal)
        );
    }

    #[test]
    fn uncovered_bins_excludes_covered_bins() {
        let mut coverage = ratio_coverage();

        coverage.sample(&2).expect("normal sample");

        assert_eq!(uncovered_names(&coverage), ["read"]);
    }

    #[test]
    fn uncovered_bins_excludes_ignore_bins() {
        assert!(!uncovered_names(&ratio_coverage()).contains(&"reset"));
    }

    #[test]
    fn uncovered_bins_excludes_illegal_bins() {
        assert!(!uncovered_names(&ratio_coverage()).contains(&"reserved"));
    }

    #[test]
    fn uncovered_bins_preserves_declaration_order() {
        assert_eq!(uncovered_names(&ratio_coverage()), ["read", "write"]);
    }

    #[test]
    fn total_sample_counter_overflow_is_atomic() {
        let mut coverage = coverage();
        coverage.samples = u64::MAX;

        assert_atomic_overflow(&mut coverage, 1, CoverageCounterKind::Samples);
    }

    #[test]
    fn ignored_sample_counter_overflow_is_atomic() {
        let mut coverage = ignore_coverage();
        coverage.ignored_samples = u64::MAX;

        assert_atomic_overflow(&mut coverage, 0, CoverageCounterKind::IgnoredSamples);
    }

    #[test]
    fn illegal_sample_counter_overflow_is_atomic() {
        let mut coverage = illegal_coverage();
        coverage.illegal_samples = u64::MAX;

        assert_atomic_overflow(&mut coverage, u8::MAX, CoverageCounterKind::IllegalSamples);
    }

    #[test]
    fn unmatched_sample_counter_overflow_is_atomic() {
        let mut coverage = coverage();
        coverage.unmatched_samples = u64::MAX;

        assert_atomic_overflow(&mut coverage, 9, CoverageCounterKind::UnmatchedSamples);
    }

    #[test]
    fn first_matching_bin_counter_overflow_is_atomic() {
        let mut coverage = overlapping_coverage();
        coverage
            .bins
            .first_mut()
            .expect("first bin")
            .set_hits(u64::MAX);

        assert_atomic_overflow(&mut coverage, 6, CoverageCounterKind::BinHits);
    }

    #[test]
    fn later_matching_bin_counter_overflow_is_atomic() {
        let mut coverage = overlapping_coverage();
        coverage.bins.first_mut().expect("first bin").set_hits(10);
        coverage
            .bins
            .get_mut(1)
            .expect("second bin")
            .set_hits(u64::MAX);

        assert_atomic_overflow(&mut coverage, 6, CoverageCounterKind::BinHits);
    }

    #[test]
    fn illegal_bin_counter_overflow_precedes_illegal_bin_error() {
        let mut coverage = illegal_coverage();
        coverage
            .bins
            .get_mut(2)
            .expect("first illegal bin")
            .set_hits(u64::MAX);
        let before = counter_snapshot(&coverage);
        let error = coverage
            .sample(&u8::MAX)
            .expect_err("overflow must fail before illegal result");

        assert_eq!(error.counter(), Some(CoverageCounterKind::BinHits));
        assert_eq!(counter_snapshot(&coverage), before);
    }

    fn coverage() -> Coverpoint<u8> {
        Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8))
            .bin(Bin::value("write", 2))
            .build()
            .expect("valid coverpoint")
    }

    fn mixed_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("mixed")
            .bin(Bin::value("normal_a", 1_u8))
            .ignore_bin(Bin::value("ignore_a", 2))
            .illegal_bin(Bin::value("illegal_a", 3))
            .bin(Bin::value("normal_b", 4))
            .build()
            .expect("valid coverpoint")
    }

    fn overlapping_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("operation")
            .bin(Bin::inclusive_range("all_operations", 0_u8, 15))
            .bin(Bin::values("writes", [4, 5, 6]))
            .bin(Bin::values("privileged", [6, 7]))
            .build()
            .expect("valid coverpoint")
    }

    fn ignore_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("value")
            .bin(Bin::inclusive_range("normal", 0_u8, 15))
            .ignore_bin(Bin::value("zero", 0))
            .build()
            .expect("valid coverpoint")
    }

    fn illegal_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("value")
            .bin(Bin::inclusive_range("normal", 0_u8, u8::MAX))
            .ignore_bin(Bin::value("ignored", u8::MAX))
            .illegal_bin(Bin::inclusive_range("illegal_a", 0xf0, u8::MAX))
            .illegal_bin(Bin::value("illegal_b", u8::MAX))
            .build()
            .expect("valid coverpoint")
    }

    fn threshold_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("threshold")
            .bin(Bin::value("twice", 1_u8).at_least(2))
            .build()
            .expect("valid coverpoint")
    }

    fn ratio_coverage() -> Coverpoint<u8> {
        Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8).at_least(2))
            .bin(Bin::value("write", 2))
            .ignore_bin(Bin::value("reset", 0))
            .illegal_bin(Bin::value("reserved", u8::MAX))
            .build()
            .expect("valid coverpoint")
    }

    fn invalid_coverpoint_name(name: &str) {
        let error = Coverpoint::builder(name)
            .bin(Bin::value("bin", 1_u8))
            .build()
            .err()
            .expect("invalid name");

        assert_eq!(
            error,
            CoverageBuildError::InvalidCoverpointName { name: name.into() }
        );
    }

    fn invalid_bin_name(bin: &str) {
        let error = Coverpoint::builder("opcode")
            .bin(Bin::value(bin, 1_u8))
            .build()
            .err()
            .expect("invalid name");

        assert_eq!(
            error,
            CoverageBuildError::InvalidBinName {
                coverpoint: "opcode".into(),
                bin: bin.into()
            }
        );
    }

    fn duplicate_name_error(builder: crate::CoverpointBuilder<u8>) {
        let error = builder.build().err().expect("duplicate name");

        assert_eq!(
            error,
            CoverageBuildError::DuplicateBinName {
                coverpoint: "opcode".into(),
                bin: "same".into()
            }
        );
    }

    fn bin_hits(coverage: &Coverpoint<u8>) -> Vec<u64> {
        coverage.bins().iter().map(CoverpointBin::hits).collect()
    }

    fn uncovered_names(coverage: &Coverpoint<u8>) -> Vec<&str> {
        coverage.uncovered_bins().map(CoverpointBin::name).collect()
    }

    fn illegal_error() -> CoverageSampleError {
        let mut coverage = illegal_coverage();

        coverage.sample(&u8::MAX).expect_err("illegal sample")
    }

    fn counter_snapshot<T>(coverpoint: &Coverpoint<T>) -> CounterSnapshot {
        CounterSnapshot {
            samples: coverpoint.samples,
            ignored_samples: coverpoint.ignored_samples,
            illegal_samples: coverpoint.illegal_samples,
            unmatched_samples: coverpoint.unmatched_samples,
            bin_hits: coverpoint.bins.iter().map(CoverpointBin::hits).collect(),
        }
    }

    fn assert_atomic_overflow(
        coverage: &mut Coverpoint<u8>,
        sampled: u8,
        expected_counter: CoverageCounterKind,
    ) {
        let before = counter_snapshot(coverage);
        let error = coverage.sample(&sampled).expect_err("counter overflow");

        assert_eq!(error.counter(), Some(expected_counter));
        assert_eq!(counter_snapshot(coverage), before);
    }
}
