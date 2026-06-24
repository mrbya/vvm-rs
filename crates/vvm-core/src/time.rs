use std::fmt;
use std::num::NonZeroU64;

/// Absolute simulator time measured in Verilator timeprecision ticks.
///
/// The value is intentionally unitless at the Rust API level. Its physical
/// meaning is determined by the HDL model's timeprecision.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SimulationTime(u64);

impl SimulationTime {
    /// Simulation time zero.
    pub const ZERO: Self = Self(0);

    /// Constructs simulation time from raw ticks.
    #[must_use]
    pub const fn from_ticks(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns its raw timeprecision ticks.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SimulationTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ticks", self.0)
    }
}

/// Non-zero amount by which the simulation time advances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TimeStep(NonZeroU64);

impl TimeStep {
    /// One simulator timeprecision tick.
    pub const ONE: Self = Self(NonZeroU64::MIN);

    /// Creates a non-zero simulation time step.
    ///
    /// # Errors
    ///
    /// Returns an error if `ticks` is zero.
    pub const fn new(ticks: u64) -> Result<Self, InvalidTimeStep> {
        match NonZeroU64::new(ticks) {
            Some(ticks) => Ok(Self(ticks)),
            None => Err(InvalidTimeStep),
        }
    }

    /// Returns the number of raw ticks.
    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.0.get()
    }
}

/// Error returned when constructing a zero time step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTimeStep;

impl fmt::Display for InvalidTimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("simulation time step must be non-zero")
    }
}

impl std::error::Error for InvalidTimeStep {}
