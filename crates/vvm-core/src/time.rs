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

    /// Adds a non-zero time step without wrapping.
    #[must_use]
    pub const fn checked_add(self, delta: TimeStep) -> Option<Self> {
        match self.0.checked_add(delta.ticks()) {
            Some(ticks) => Some(Self(ticks)),
            None => None,
        }
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

    /// Adds a non-zero time step without wrapping.
    #[must_use]
    pub const fn checked_add(self, delta: Self) -> Option<Self> {
        match self.0.checked_add(delta.ticks()) {
            Some(ticks) => Some(Self(ticks)),
            None => None,
        }
    }
}

/// Error returned when constructing a zero time step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTimeStep;

impl fmt::Display for InvalidTimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("simulation time step must be non-zero")
    }
}

impl std::error::Error for InvalidTimeStep {}

/// Timing configuration for one complete synchronous clock cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleTiming {
    /// Duration of the inactive clock phase.
    inactive_phase: TimeStep,

    /// Duration of the active clock phase.
    active_phase: TimeStep,
}

impl CycleTiming {
    /// One tick per inactive and active phase.
    pub const UNIT: Self = Self::new(TimeStep::ONE, TimeStep::ONE);

    /// Creates a new asymetric cycle timing configuration.
    #[must_use]
    pub const fn new(inactive_phase: TimeStep, active_phase: TimeStep) -> Self {
        Self {
            inactive_phase,
            active_phase,
        }
    }

    /// Constructs default cycle timing (symetric, 1 time step).
    #[must_use]
    pub const fn default() -> Self {
        Self::UNIT
    }

    /// Creates a symetric cycle from one half-period.
    #[must_use]
    pub const fn symetric(half_period: TimeStep) -> Self {
        Self::new(half_period, half_period)
    }

    /// Returns the inactive clock-phase duration.
    #[must_use]
    pub const fn inactive_phase(self) -> TimeStep {
        self.inactive_phase
    }

    /// Returns the active clock-phase duration.
    #[must_use]
    pub const fn active_phase(self) -> TimeStep {
        self.active_phase
    }
}

#[cfg(test)]
mod tests {
    use super::{CycleTiming, InvalidTimeStep, SimulationTime, TimeStep};

    #[test]
    fn rejects_zero_time_step() {
        assert!(matches!(TimeStep::new(0), Err(InvalidTimeStep)));
    }

    #[test]
    fn simulation_time_adds_non_zero_step() -> Result<(), InvalidTimeStep> {
        let time = SimulationTime::from_ticks(7);
        let delta = TimeStep::new(5)?;

        assert_eq!(
            time.checked_add(delta),
            Some(SimulationTime::from_ticks(12))
        );

        Ok(())
    }

    #[test]
    fn simulation_time_rejects_overflow() {
        assert_eq!(
            SimulationTime::from_ticks(u64::MAX).checked_add(TimeStep::ONE),
            None
        );
    }

    #[test]
    fn unit_cycle_timing_uses_one_tick_per_phase() {
        let timing = CycleTiming::UNIT;

        assert_eq!(timing.inactive_phase(), TimeStep::ONE);
        assert_eq!(timing.active_phase(), TimeStep::ONE);
    }
}
