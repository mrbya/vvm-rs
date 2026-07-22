use std::fmt;
use std::num::NonZeroU64;

use crate::coverage::matcher::BinMatcher;

/// Semantic role of one functional coverage bin.
///
/// [`BinKind::Normal`] contributes to coverage. [`BinKind::Ignore`] excludes a
/// matching sample from normal coverage. [`BinKind::Illegal`] records invalid
/// behavior and causes sampling to return an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinKind {
    /// A normal bin contributing to functional coverage.
    Normal,

    /// A bin whose matching samples are intentionally excluded.
    Ignore,

    /// A bin whose matching samples represent invalid behaviour.
    Illegal,
}

impl fmt::Display for BinKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Normal => f.write_str("normal"),
            Self::Ignore => f.write_str("ignore"),
            Self::Illegal => f.write_str("illegal"),
        }
    }
}

/// Opaque identifier of one bin within its owning coverpoint.
///
/// IDs are assigned in global declaration order, are local to one coverpoint,
/// and must not be interpreted as global or persistent identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinId(u32);

impl BinId {
    /// Creates a coverpoint-local bin identifier.
    ///
    /// This constructor is primarily useful for comparing IDs returned by a
    /// coverpoint; the ordinal is not a global persistent identity.
    #[must_use]
    pub const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// Returns the coverpoint-local declaration ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }
}

/// Declarative definition of one functional coverage bin.
///
/// A definition has no runtime counter until a [`crate::CoverpointBuilder`]
/// validates and binds it into a [`CoverpointBin`]. [`Bin::value`] and
/// [`Bin::values`] require `PartialEq`; [`Bin::inclusive_range`] requires
/// `PartialOrd`. [`Bin::at_least`] is validated by builder [`build`][crate::CoverpointBuilder::build]:
/// zero is invalid and custom thresholds apply only to normal bins.
pub struct Bin<T> {
    /// Stable user-provided name.
    name: String,

    /// Declarative matching rule.
    matcher: BinMatcher<T>,

    /// Requested hit threshold.
    required_hits: u64,
}

impl<T> Bin<T> {
    /// Creates a bin matching one exact value.
    #[must_use]
    pub fn value(name: impl Into<String>, expected: T) -> Self
    where
        T: PartialEq,
    {
        Self {
            name: name.into(),
            matcher: BinMatcher::value(expected),
            required_hits: 1,
        }
    }

    /// Creates a bin matching any value from one set.
    #[must_use]
    pub fn values(name: impl Into<String>, expected: impl IntoIterator<Item = T>) -> Self
    where
        T: PartialEq,
    {
        Self {
            name: name.into(),
            matcher: BinMatcher::values(expected),
            required_hits: 1,
        }
    }

    /// Creates a bin matching one inclusive range.
    #[must_use]
    pub fn inclusive_range(name: impl Into<String>, start: T, end: T) -> Self
    where
        T: PartialOrd,
    {
        Self {
            name: name.into(),
            matcher: BinMatcher::inclusive_range(start, end),
            required_hits: 1,
        }
    }

    /// Sets the requested normal-bin hit requirement.
    ///
    /// The builder rejects zero and rejects custom thresholds on ignore or
    /// illegal bins.
    #[must_use]
    pub const fn at_least(mut self, required_hits: u64) -> Self {
        self.required_hits = required_hits;
        self
    }

    /// Returns the bin name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the requested hit requirement.
    pub const fn requested_hits(&self) -> u64 {
        self.required_hits
    }

    /// Returns the declarative matcher for coverpoint validation and sampling.
    pub(crate) const fn matcher(&self) -> &BinMatcher<T> {
        &self.matcher
    }
}

/// Validated bin bound to a live coverpoint.
///
/// It exposes its kind, coverpoint-local ID, hit threshold, and runtime hit
/// count. [`Self::covered`] is meaningful only for normal bins; ignore and
/// illegal bins always report uncovered.
pub struct CoverpointBin<T> {
    /// Deterministic coverpoint-local identifier.
    id: BinId,

    /// Semantic bin role.
    kind: BinKind,

    /// Coverpoint bin definition.
    definition: Bin<T>,

    /// Validated non-zero hit threshold.
    required_hits: NonZeroU64,

    /// Runtime hit count.
    hits: u64,
}

impl<T> CoverpointBin<T> {
    /// Creates a validated live bin.
    pub const fn new(
        id: BinId,
        kind: BinKind,
        definition: Bin<T>,
        required_hits: NonZeroU64,
    ) -> Self {
        Self {
            id,
            kind,
            definition,
            required_hits,
            hits: 0,
        }
    }

    /// Returns the local identifier.
    #[must_use]
    pub const fn id(&self) -> BinId {
        self.id
    }

    /// Returns the bin name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.definition.name()
    }

    /// Returns the bin kind.
    #[must_use]
    pub const fn kind(&self) -> BinKind {
        self.kind
    }

    /// Returns the required hit count.
    #[must_use]
    pub const fn required_hits(&self) -> u64 {
        self.required_hits.get()
    }

    /// Returns the current hit count.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }

    /// Returns whether this normal bin is covered.
    #[must_use]
    pub const fn covered(&self) -> bool {
        matches!(self.kind, BinKind::Normal) && self.hits >= self.required_hits.get()
    }

    /// Returns whether the bin matches a value.
    pub fn matches(&self, sampled: &T) -> bool {
        self.definition.matcher().matches(sampled)
    }

    /// Returns the next hit count.
    pub const fn checked_next_hits(&self) -> Option<u64> {
        self.hits.checked_add(1)
    }

    /// Replaces the hit counter with a prepared value.
    pub const fn set_hits(&mut self, hits: u64) {
        self.hits = hits;
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use crate::{Bin, BinId, BinKind, Coverpoint, CoverpointBin};

    #[test]
    fn formats_normal_bin_kind() {
        assert_eq!(BinKind::Normal.to_string(), "normal");
    }

    #[test]
    fn formats_ignore_bin_kind() {
        assert_eq!(BinKind::Ignore.to_string(), "ignore");
    }

    #[test]
    fn formats_illegal_bin_kind() {
        assert_eq!(BinKind::Illegal.to_string(), "illegal");
    }

    #[test]
    fn bin_ids_preserve_declaration_order() {
        let coverage = Coverpoint::builder("kind_order")
            .bin(Bin::value("normal_a", 1_u8))
            .ignore_bin(Bin::value("ignore_a", 2))
            .illegal_bin(Bin::value("illegal_a", 3))
            .bin(Bin::value("normal_b", 4))
            .build()
            .expect("valid coverpoint");
        let ids = coverage
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
    fn live_bin_starts_with_zero_hits() {
        let bin = CoverpointBin::new(
            BinId::new(0),
            BinKind::Normal,
            Bin::value("one", 1_u8),
            NonZeroU64::MIN,
        );

        assert_eq!(bin.hits(), 0);
    }

    #[test]
    fn normal_bin_is_uncovered_before_threshold() {
        let bin = normal_bin(2, 0);

        assert!(!bin.covered());
    }

    #[test]
    fn normal_bin_remains_uncovered_below_threshold() {
        let bin = normal_bin(2, 1);

        assert!(!bin.covered());
    }

    #[test]
    fn normal_bin_is_covered_at_threshold() {
        let bin = normal_bin(2, 2);

        assert!(bin.covered());
    }

    #[test]
    fn normal_bin_remains_covered_above_threshold() {
        let bin = normal_bin(2, 3);

        assert!(bin.covered());
    }

    #[test]
    fn ignore_bin_never_reports_covered() {
        let mut bin = CoverpointBin::new(
            BinId::new(0),
            BinKind::Ignore,
            Bin::value("ignored", 1_u8),
            NonZeroU64::MIN,
        );
        bin.set_hits(1);

        assert!(!bin.covered());
    }

    #[test]
    fn illegal_bin_never_reports_covered() {
        let mut bin = CoverpointBin::new(
            BinId::new(0),
            BinKind::Illegal,
            Bin::value("illegal", 1_u8),
            NonZeroU64::MIN,
        );
        bin.set_hits(1);

        assert!(!bin.covered());
    }

    fn normal_bin(required_hits: u64, hits: u64) -> CoverpointBin<u8> {
        let required_hits = NonZeroU64::new(required_hits).expect("non-zero threshold");
        let mut bin = CoverpointBin::new(
            BinId::new(0),
            BinKind::Normal,
            Bin::value("normal", 1_u8),
            required_hits,
        );
        bin.set_hits(hits);

        bin
    }
}
