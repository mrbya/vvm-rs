use crate::{
    CoverageDefinitionError, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
    CoverageRuntimeError, ObservedCycle,
};

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

#[cfg(test)]
mod tests {
    use super::{CoverageInstance, CoverageSpec};
    use crate::CoverageDefinitionError;

    struct EmptyModel;

    impl CoverageSpec for EmptyModel {
        const DEFINITION_NAME: &'static str = "decoder";
        const DEFINITION_REVISION: u64 = 2;

        fn visit_coverage_items(&self, _visitor: &mut dyn crate::CoverageGroupVisitor) {}
    }

    #[test]
    fn coverage_instance_reports_invalid_group_identity() {
        let error = CoverageInstance::__vvm_new("dut..decoder", EmptyModel)
            .map(|_| ())
            .expect_err("invalid instance paths must be rejected");

        assert!(matches!(
            error,
            CoverageDefinitionError::Group { ref source }
                if matches!(
                    source,
                    crate::CoverageGroupError::InvalidInstancePath { path } if path == "dut..decoder"
                )
        ));
    }

    #[test]
    fn coverage_instance_validates_its_model_items() {
        let error = CoverageInstance::__vvm_new("dut.decoder", EmptyModel)
            .map(|_| ())
            .expect_err("coverage models must expose at least one item");

        assert!(matches!(
            error,
            CoverageDefinitionError::Group { ref source }
                if matches!(source, crate::CoverageGroupError::EmptyGroup { .. })
        ));
    }
}
