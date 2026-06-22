use crate::Dut;

/// A stimulus value that can drive a particular DUT,
///
/// Driving changes DUT inputs only. It does not evaluate the DUT or
/// advance simulation time.
pub trait Drive<D>
where
    D: Dut,
{
    /// Drives stimuli to DUT.
    ///
    /// # Errors
    ///
    /// Returns the DUT error when an input is failed to be driven.
    fn drive(&self, dut: &mut D) -> Result<(), D::Error>;
}
