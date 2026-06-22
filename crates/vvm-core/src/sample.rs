use crate::Dut;

/// An observation sampled from a particular DUT.
///
/// Sampling reads current state only. It does not evaluate the DUT
/// or advance simulation time.
pub trait Sample<D>: Sized
where
    D: Dut,
{
    /// Samples the current DUT outputs.
    ///
    /// # Errors
    ///
    /// Returns the DUT error when an output cannot be sampled.
    fn sample(dut: &D) -> Result<Self, D::Error>;
}
