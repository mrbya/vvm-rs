use crate::Dut;

/// Drive the inactive and active phases of a synchronous DUT clock.
///
/// Clock driving changes only the clock input. Evaluation remains the
/// responsibility of the testbench runner.
pub trait Clock<D>
where
    D: Dut,
{
    /// Drives the clock to its inactive phase.
    ///
    /// # Errors
    ///
    /// Returns the DUT error when the clock cannot be driven.
    fn drive_inactive(&mut self, dut: &mut D) -> Result<(), D::Error>;

    /// Drives the clock to its active phase.
    ///
    /// # Errors
    ///
    /// Returns the DUT error when the clock cannot be driven.
    fn drive_active(&mut self, dut: &mut D) -> Result<(), D::Error>;
}
