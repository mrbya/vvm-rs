use std::collections::BTreeSet;
use std::num::NonZeroU64;

use crate::coverage::{
    Bin, BinId, BinKind, CoverageBuildError, CoverageSampleError, CoverpointBin,
    MatcherValidationError,
};

/// Bin defined waiting to be validated and bound.
struct PendingBin<T> {
    /// Requested semantic role.
    kind: BinKind,

    /// User-provided definition.
    definition: Bin<T>,
}

/// Builder for one typed functional coverpoint.
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

/// Explicitly owned typed functional coverpoint.
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
    /// Start constructing a coverpoint.
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
    pub fn bins(&self) -> impl ExactSizeIterator<Item = &CoverpointBin<T>> {
        self.bins.iter()
    }

    /// Samples one value into the coverpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when an illegal bin matches or a coverage
    /// counter cannot be incremented.
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

/// Disposition of one successfully processed sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageSampleDisposition {
    /// One or more normal bins were hit.
    Hit,

    /// An ignore bin excluded the sample.
    Ignored,

    /// No bin matched.
    Unmatched,
}

/// Result of one non-illegal coverpoint sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverpointSample {
    /// Selected disposition.
    disposition: CoverageSampleDisposition,

    /// Matching normal bins.
    matched_bins: Box<[BinId]>,
}

impl CoverpointSample {
    /// Creates a normal-hit result.
    const fn new_hit(matched_bins: Box<[BinId]>) -> Self {
        Self {
            disposition: CoverageSampleDisposition::Hit,
            matched_bins,
        }
    }

    /// Creates an ignored result.
    fn new_ignored() -> Self {
        Self {
            disposition: CoverageSampleDisposition::Ignored,
            matched_bins: Box::default(),
        }
    }

    /// Creates an unmatched result.
    fn new_unmatched() -> Self {
        Self {
            disposition: CoverageSampleDisposition::Unmatched,
            matched_bins: Box::default(),
        }
    }

    /// Returns the disposition.
    #[must_use]
    pub const fn disposition(&self) -> CoverageSampleDisposition {
        self.disposition
    }

    /// Returns matching normal-bin identifiers.
    #[must_use]
    pub fn matched_bins(&self) -> &[BinId] {
        &self.matched_bins
    }

    /// Returns whether normal bins were hit.
    #[must_use]
    pub const fn hit(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Hit)
    }

    /// Returns whether the sample was ignored.
    #[must_use]
    pub const fn ignored(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Ignored)
    }

    /// Returns whether no bin matched.
    #[must_use]
    pub const fn unmatched(&self) -> bool {
        matches!(self.disposition, CoverageSampleDisposition::Unmatched)
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
