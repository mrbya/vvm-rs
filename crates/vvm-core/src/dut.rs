use crate::{SimulationTime, TimeStep};

/// A simulation model that can be evaluated and finalized.
///
/// Implementations may wrap a Verilated model, another simulator, or a
/// completely pure-Rust mock.
pub trait Dut {
    /// Error returned by DUT lifecycle operations.
    type Error;

    /// Evaluates the current DUT state.
    ///
    /// This operation does not implicitly advance time or toggle clocks.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error when evaluation canot complete.
    fn evaluate(&mut self) -> Result<(), Self::Error>;

    /// Returns the current absolute simulation time.
    ///
    /// This remains available after the DUT has been finalized.
    fn simulation_time(&self) -> SimulationTime;

    /// Advances the DUT simulation time.
    ///
    /// This operation does not evaluate the DUT or change any signal.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error if time cannot be advanced,
    /// including arithmetic overflow or use after finalization.
    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error>;

    /// Finalizes the DUT.
    ///
    /// Implementations must permit repeated calls without finalizing the
    /// underlying model more than once.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error when finalization cannot complete.
    fn finalize(&mut self) -> Result<(), Self::Error>;
}
