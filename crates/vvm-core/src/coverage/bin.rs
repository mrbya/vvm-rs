use std::fmt;
use std::num::NonZeroU64;

use crate::coverage::BinMatcher;

/// Semantic role of one functional coverage bin.
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

/// Identifier of one bin within its owning coverpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinId(u32);

impl BinId {
    /// Creates a coverpoint-local bin identifier.
    #[must_use]
    pub const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }
}

/// Declarative definition of one functional coverage bin.
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

    /// Sets the normal-bin hit requirement.
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

    /// Returns the declarative matcher.
    pub const fn matcher(&self) -> &BinMatcher<T> {
        &self.matcher
    }
}

/// Validated bin bounded to a live coverpoint.
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
