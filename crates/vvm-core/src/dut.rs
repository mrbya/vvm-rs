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

/// A DUT exposing internally scheduled timing events.
///
/// Timing-enabled DUTs report absolute future simulation times. Querying the
/// event queue does not advance simulation time or evaluate the model.
/// Implementations must return `None` from [`Self::next_time_slot`] when no
/// delayed event is pending.
pub trait TimedDut: Dut {
    /// Returns whether delayed events remain pending.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined DUT error when the delayed-event
    /// queue cannot be queried.
    fn events_pending(&self) -> Result<bool, Self::Error>;

    /// Returns the absolute time of the next delayed event.
    ///
    /// Returns `None` when no delayed event is pending. This operation must
    /// not advance simulation time or evaluate the DUT.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined DUT error when the event time cannot
    /// be queried.
    fn next_time_slot(&self) -> Result<Option<SimulationTime>, Self::Error>;
}

/// Optional waveform-tracing lifecycle for DUTs that support generated traces.
pub trait TraceableDut: Dut {
    /// Opens a waveform trace for the DUT.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error if tracing cannot be configured.
    fn open_trace(&mut self, path: &std::path::Path) -> Result<(), Self::Error>;

    /// Flushes and closes the active waveform trace.
    ///
    /// # Errors
    ///
    /// Returns an implementation-defined error if the trace cannot be closed.
    fn close_trace(&mut self) -> Result<(), Self::Error>;

    /// Returns whether the waveform trace is currently open.
    fn trace_is_open(&self) -> bool;
}
