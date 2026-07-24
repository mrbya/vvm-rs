use std::error::Error;

use crate::{
    CoverageBuildError, CoverageGroup, CoverageGroupError, CoverageGroupInstance,
    CoverageGroupVisitor, CoverageSampleError, CrossBuildError, CrossSampleError, ObservedCycle,
    SimulationTime,
};

/// Failure while constructing one typed functional-coverage model.
#[derive(Debug)]
pub enum CoverageDefinitionError {
    /// One generated coverpoint field failed to build.
    Coverpoint {
        /// Declared field and item name.
        item: &'static str,
        /// Underlying coverpoint build error.
        source: CoverageBuildError,
    },
    /// One generated cross field failed to build.
    Cross {
        /// Declared field and item name.
        item: &'static str,
        /// Underlying cross build error.
        source: CrossBuildError,
    },
    /// Group identity or completed-group validation failed.
    Group {
        /// Underlying coverage-group error.
        source: CoverageGroupError,
    },
}

impl CoverageDefinitionError {
    /// Returns the declared coverage item when construction reached one.
    #[must_use]
    pub const fn item(&self) -> Option<&'static str> {
        match *self {
            Self::Coverpoint { item, .. } | Self::Cross { item, .. } => Some(item),
            Self::Group { .. } => None,
        }
    }

    /// Returns the underlying coverpoint build error when applicable.
    #[must_use]
    pub const fn coverpoint_source(&self) -> Option<&CoverageBuildError> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { .. } | Self::Group { .. } => None,
        }
    }

    /// Returns the underlying cross build error when applicable.
    #[must_use]
    pub const fn cross_source(&self) -> Option<&CrossBuildError> {
        match *self {
            Self::Cross { ref source, .. } => Some(source),
            Self::Coverpoint { .. } | Self::Group { .. } => None,
        }
    }

    /// Returns the underlying group error when applicable.
    #[must_use]
    pub const fn group_source(&self) -> Option<&CoverageGroupError> {
        match *self {
            Self::Group { ref source } => Some(source),
            Self::Coverpoint { .. } | Self::Cross { .. } => None,
        }
    }
}

impl std::fmt::Display for CoverageDefinitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Coverpoint { item, ref source } => {
                write!(
                    formatter,
                    "failed to build coverage coverpoint `{item}`: {source}"
                )
            }
            Self::Cross { item, ref source } => {
                write!(
                    formatter,
                    "failed to build coverage cross `{item}`: {source}"
                )
            }
            Self::Group { ref source } => {
                write!(formatter, "failed to construct coverage group: {source}")
            }
        }
    }
}

impl Error for CoverageDefinitionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { ref source, .. } => Some(source),
            Self::Group { ref source } => Some(source),
        }
    }
}

/// Kind of coverage item involved in a runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageRuntimeItemKind {
    /// Coverpoint sampling.
    Coverpoint,
    /// Cross sampling.
    Cross,
}

/// Failure while sampling one typed functional-coverage model.
#[derive(Debug)]
pub enum CoverageRuntimeError {
    /// One coverpoint failed for an observed transaction.
    Coverpoint {
        /// Declared coverage item name.
        item: &'static str,
        /// Zero-based transaction cycle.
        cycle: u64,
        /// Observation sampling time.
        time: SimulationTime,
        /// Underlying coverpoint error.
        source: CoverageSampleError,
    },
    /// One cross failed for an observed transaction.
    Cross {
        /// Declared coverage item name.
        item: &'static str,
        /// Zero-based transaction cycle.
        cycle: u64,
        /// Observation sampling time.
        time: SimulationTime,
        /// Underlying cross error.
        source: CrossSampleError,
    },
}

impl CoverageRuntimeError {
    /// Returns the declared coverage item name.
    #[must_use]
    pub const fn item(&self) -> &'static str {
        match *self {
            Self::Coverpoint { item, .. } | Self::Cross { item, .. } => item,
        }
    }

    /// Returns the zero-based observed transaction cycle.
    #[must_use]
    pub const fn cycle(&self) -> u64 {
        match *self {
            Self::Coverpoint { cycle, .. } | Self::Cross { cycle, .. } => cycle,
        }
    }

    /// Returns the observation sampling time.
    #[must_use]
    pub const fn time(&self) -> SimulationTime {
        match *self {
            Self::Coverpoint { time, .. } | Self::Cross { time, .. } => time,
        }
    }

    /// Returns the coverage item kind.
    #[must_use]
    pub const fn item_kind(&self) -> CoverageRuntimeItemKind {
        match *self {
            Self::Coverpoint { .. } => CoverageRuntimeItemKind::Coverpoint,
            Self::Cross { .. } => CoverageRuntimeItemKind::Cross,
        }
    }

    /// Returns the underlying coverpoint sampling error when applicable.
    #[must_use]
    pub const fn coverpoint_source(&self) -> Option<&CoverageSampleError> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { .. } => None,
        }
    }

    /// Returns the underlying cross sampling error when applicable.
    #[must_use]
    pub const fn cross_source(&self) -> Option<&CrossSampleError> {
        match *self {
            Self::Cross { ref source, .. } => Some(source),
            Self::Coverpoint { .. } => None,
        }
    }
}

impl std::fmt::Display for CoverageRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Coverpoint {
                item,
                cycle,
                time,
                ref source,
            } => write!(
                formatter,
                "coverage coverpoint `{item}` failed at cycle {cycle} at {time}: {source}"
            ),
            Self::Cross {
                item,
                cycle,
                time,
                ref source,
            } => write!(
                formatter,
                "coverage cross `{item}` failed at cycle {cycle} at {time}: {source}"
            ),
        }
    }
}

impl Error for CoverageRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::Coverpoint { ref source, .. } => Some(source),
            Self::Cross { ref source, .. } => Some(source),
        }
    }
}

/// Typed functional coverage sampled from observed transactions.
pub trait CoverageModel<S, O>: CoverageGroup {
    /// Samples one successfully observed transaction.
    ///
    /// # Errors
    ///
    /// Returns an item-aware functional-coverage runtime error.
    fn sample(&mut self, cycle: ObservedCycle<'_, S, O>) -> Result<(), CoverageRuntimeError>;
}

/// Generated coverage definition metadata and deterministic visitation.
#[doc(hidden)]
pub trait CoverageSpec {
    /// Stable group definition name.
    const DEFINITION_NAME: &'static str;
    /// Semantic group definition revision.
    const DEFINITION_REVISION: u64;
    /// Visits coverage fields in stable definition order.
    fn visit_coverage_items(&self, visitor: &mut dyn CoverageGroupVisitor);
}

/// Generated typed coverage sampling wiring.
#[doc(hidden)]
pub trait CoverageSampleSpec<S, O>: CoverageSpec {
    /// Samples generated coverpoints and crosses.
    fn sample_coverage_items(
        &mut self,
        cycle: ObservedCycle<'_, S, O>,
    ) -> Result<(), CoverageRuntimeError>;
}

/// One typed coverage model bound to stable group-instance metadata.
pub struct CoverageInstance<M> {
    /// Stable definition and hierarchical instance identity.
    instance: CoverageGroupInstance,
    /// User-defined typed coverage model.
    model: M,
}

impl<M> CoverageInstance<M> {
    /// Returns stable group-instance metadata.
    #[must_use]
    pub const fn group_instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    /// Returns the typed coverage model.
    #[must_use]
    pub const fn model(&self) -> &M {
        &self.model
    }

    /// Returns mutable typed coverage state.
    #[must_use]
    pub const fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Consumes the wrapper and returns the model.
    #[must_use]
    pub fn into_model(self) -> M {
        self.model
    }
}

impl<M> CoverageInstance<M>
where
    M: CoverageSpec,
{
    /// Constructs and validates generated coverage with a stable instance path.
    #[doc(hidden)]
    pub fn __vvm_new(
        instance_path: impl Into<String>,
        model: M,
    ) -> Result<Self, CoverageDefinitionError> {
        let instance = CoverageGroupInstance::new_with_revision(
            M::DEFINITION_NAME,
            instance_path,
            M::DEFINITION_REVISION,
        )
        .map_err(|source| CoverageDefinitionError::Group { source })?;
        let coverage = Self { instance, model };

        coverage
            .validate()
            .map_err(|source| CoverageDefinitionError::Group { source })?;

        Ok(coverage)
    }
}

impl<M> CoverageGroup for CoverageInstance<M>
where
    M: CoverageSpec,
{
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        self.model.visit_coverage_items(visitor);
    }
}

impl<M, S, O> CoverageModel<S, O> for CoverageInstance<M>
where
    M: CoverageSampleSpec<S, O>,
{
    fn sample(&mut self, cycle: ObservedCycle<'_, S, O>) -> Result<(), CoverageRuntimeError> {
        self.model.sample_coverage_items(cycle)
    }
}
