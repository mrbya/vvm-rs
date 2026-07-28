use crate::{CycleTiming, Dut, TimeStep};

/// Independently timed clock scheduling primitives.
pub mod scheduler;

/// Clock scheduler configuration errors.
mod error;

pub use error::ClockConfigurationError;
pub use scheduler::ClockScheduler;

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

/// Timing configuration for one independently scheduled clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockTiming {
    /// Recurring inactive and active phase durations.
    cycle: CycleTiming,

    /// Delay from scheduler initialization to the first active transition.
    first_active_after: TimeStep,
}

impl ClockTiming {
    /// Unit clock timing compatible with the existing single-clock runner.
    pub const UNIT: Self = Self::from_cycle(CycleTiming::UNIT);

    /// Creates an independent clock timing configuration.
    #[must_use]
    pub const fn new(cycle: CycleTiming, first_active_after: TimeStep) -> Self {
        Self {
            cycle,
            first_active_after,
        }
    }

    /// Creates timing whose first active transition follows one normal inactive phase.
    #[must_use]
    pub const fn from_cycle(cycle: CycleTiming) -> Self {
        Self::new(cycle, cycle.inactive_phase())
    }

    /// Returns recurring cycle timing.
    #[must_use]
    pub const fn cycle(self) -> CycleTiming {
        self.cycle
    }

    /// Returns the delay to the first active transition.
    #[must_use]
    pub const fn first_active_after(self) -> TimeStep {
        self.first_active_after
    }

    /// Returns recurring inactive-phase duration.
    #[must_use]
    pub const fn inactive_phase(self) -> TimeStep {
        self.cycle.inactive_phase()
    }

    /// Returns recurring active-phase duration.
    #[must_use]
    pub const fn active_phase(self) -> TimeStep {
        self.cycle.active_phase()
    }
}

impl Default for ClockTiming {
    fn default() -> Self {
        Self::UNIT
    }
}

#[cfg(test)]
mod tests {
    use crate::{ClockTiming, CycleTiming, InvalidTimeStep, TimeStep};

    #[test]
    fn unit_clock_timing_matches_cycle_unit() {
        assert_eq!(ClockTiming::UNIT.cycle(), CycleTiming::UNIT);
    }

    #[test]
    fn default_clock_timing_is_unit() {
        assert_eq!(ClockTiming::default(), ClockTiming::UNIT);
    }

    #[test]
    fn from_cycle_uses_inactive_phase_for_first_delay() -> Result<(), InvalidTimeStep> {
        let cycle = CycleTiming::new(TimeStep::new(3)?, TimeStep::new(2)?);
        let timing = ClockTiming::from_cycle(cycle);

        assert_eq!(timing.first_active_after(), TimeStep::new(3)?);
        assert_eq!(timing.active_phase(), TimeStep::new(2)?);

        Ok(())
    }

    #[test]
    fn custom_first_active_delay_is_preserved() -> Result<(), InvalidTimeStep> {
        let timing = ClockTiming::new(CycleTiming::UNIT, TimeStep::new(7)?);

        assert_eq!(timing.first_active_after(), TimeStep::new(7)?);

        Ok(())
    }

    #[test]
    fn clock_timing_exposes_recurring_inactive_phase() -> Result<(), InvalidTimeStep> {
        let timing = ClockTiming::from_cycle(CycleTiming::new(TimeStep::new(3)?, TimeStep::ONE));

        assert_eq!(timing.inactive_phase(), TimeStep::new(3)?);

        Ok(())
    }

    #[test]
    fn clock_timing_exposes_recurring_active_phase() -> Result<(), InvalidTimeStep> {
        let timing = ClockTiming::from_cycle(CycleTiming::new(TimeStep::ONE, TimeStep::new(3)?));

        assert_eq!(timing.active_phase(), TimeStep::new(3)?);

        Ok(())
    }
}
