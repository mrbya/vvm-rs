/// Predicts expected behavior from applied stimuli.
///
/// A reference model may retain internal state between predictions.
pub trait ReferenceModel<S> {
    /// Expected value predicted by the model.
    type Expected;

    /// Updates the model from one stimulus and returns the expected result.
    fn predict(&mut self, stimulus: &S) -> Self::Expected;
}
