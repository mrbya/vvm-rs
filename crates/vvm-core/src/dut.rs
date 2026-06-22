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
    fn eval(&mut self) -> Result<(), Self::Error>;

    /// Finalizes the DUT.
    ///
    /// Implementations must permit repeated calls without finalizing the
    /// underlying model more than once.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error when finalization cannot complete.
    fn finish(&mut self) -> Result<(), Self::Error>;
}
