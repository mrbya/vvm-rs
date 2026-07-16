use std::fmt;

use crate::{Clock, CycleTiming, Dut, TimeStep};

/// Timing configuration for one independently scheduled clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockTiming {
    /// Recurring inactive and active phase durations.
    cycle: CycleTiming,

    /// Delay for scheduler initialization to the first active transition.
    first_active_after: TimeStep,
}

impl ClockTiming {
    /// Unit clock timing compatible with a simple, single-clock runner.
    pub const UNIT: Self = Self::from_cycle(CycleTiming::UNIT);

    /// Creates an independent clock timing configuration.
    #[must_use]
    pub const fn new(cycle: CycleTiming, first_active_after: TimeStep) -> Self {
        Self {
            cycle,
            first_active_after,
        }
    }

    /// Creates timing whose first active transition occurst after one normal inactive phase.
    #[must_use]
    pub const fn from_cycle(cycle: CycleTiming) -> Self {
        Self {
            cycle,
            first_active_after: cycle.inactive_phase(),
        }
    }

    /// Returns the recurring cycle timing.
    #[must_use]
    pub const fn cycle(self) -> CycleTiming {
        self.cycle
    }

    /// Returns the delay to the 1st active transition.
    #[must_use]
    pub const fn first_active_after(self) -> TimeStep {
        self.first_active_after
    }

    /// Returns the recurring inactive phase duration.
    #[must_use]
    pub const fn inactive_phase(self) -> TimeStep {
        self.cycle.inactive_phase()
    }

    /// Returns the recurring active phase duration.
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

/// Error returned while configuring a clock scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClockConfigurationError {
    /// A clock name was empty.
    EmptyName,

    /// Two clocks used the same diagnostic name.
    DuplicateName {
        /// Duplicate clock name.
        name: String,
    },
}

impl fmt::Display for ClockConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EmptyName => f.write_str("clock name must not be empty"),

            Self::DuplicateName { ref name } => {
                write!(f, "clock name `{name}` is already registered")
            }
        }
    }
}

impl std::error::Error for ClockConfigurationError {}

/// Current semantic clock phase.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ClockPhase {
    /// Clock is in its configured inactive level.
    #[default]
    Inactive,

    /// Clock is in its configured active level.
    Active,
}

impl ClockPhase {
    /// Returns the next semantic phase semantic phase.
    const fn next(self) -> Self {
        match self {
            Self::Inactive => Self::Active,
            Self::Active => Self::Inactive,
        }
    }
}

/// Concrete clock configuration and state.
struct ClockDomain<'clock, D>
where
    D: Dut,
{
    /// Diagnostic clock name.
    name: String,

    /// Concrete clock driver.
    driver: Box<dyn Clock<D> + 'clock>,

    /// Timing configuration.
    timing: ClockTiming,

    /// Current semantic clock phase.
    phase: ClockPhase,

    /// Remaining time until the next transition.
    remaining: TimeStep,
}

impl<'clock, D> ClockDomain<'clock, D>
where
    D: Dut,
{
    fn new<C>(name: String, driver: C, timing: ClockTiming) -> Self
    where
        C: Clock<D> + 'clock,
    {
        Self {
            name,
            driver: Box::new(driver),
            timing,
            phase: ClockPhase::Inactive,
            remaining: timing.first_active_after(),
        }
    }
}

pub struct ClockScheduler<'clock, D>
where
    D: Dut,
{
    /// Required primary domain.
    primary: ClockDomain<'clock, D>,

    /// Secondary domains in registration order.
    secondary: Vec<ClockDomain<'clock, D>>,
}

impl<'clock, D> ClockScheduler<'clock, D>
where
    D: Dut,
{
    /// Creates a scheduler with one required primary clock.
    ///
    /// The primary clock defines transaction boundaries once the scheduler is
    /// integrated into [`Testbench`].
    ///
    /// # Errors
    ///
    /// Returns [`ClockConfigurationError::EmptyName`] when `name` is empty.
    pub fn new<C>(
        name: impl Into<String>,
        clock: C,
        timing: ClockTiming,
    ) -> Result<Self, ClockConfigurationError>
    where
        C: Clock<D> + 'clock,
    {
        let name = name.into();
        validate_clock_name(&name)?;

        Ok(Self {
            primary: ClockDomain::new(name, clock, timing),
            secondary: Vec::new(),
        })
    }

    /// Adds one secondary clock in registration order.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is empty or already registered.
    pub fn with_clock<C>(
        mut self,
        name: impl Into<String>,
        clock: C,
        timing: ClockTiming,
    ) -> Result<Self, ClockConfigurationError>
    where
        C: Clock<D> + 'clock,
    {
        self.add_clock(name, clock, timing)?;
        Ok(self)
    }

    /// Registers one secondary clock.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is empty or already registered.
    pub fn add_clock<C>(
        &mut self,
        name: impl Into<String>,
        clock: C,
        timing: ClockTiming,
    ) -> Result<(), ClockConfigurationError>
    where
        C: Clock<D> + 'clock,
    {
        let name = name.into();

        validate_clock_name(&name)?;

        if self.contains_name(&name) {
            return Err(ClockConfigurationError::DuplicateName { name });
        }

        self.secondary.push(ClockDomain::new(name, clock, timing));

        Ok(())
    }

    /// Checks whether the scheduler already contains a clock with a given diagnostic name.
    fn contains_name(&self, name: &str) -> bool {
        self.primary.name == name || self.secondary.iter().any(|domain| domain.name == name)
    }
}

/// Validates a diagnostic clock name.
const fn validate_clock_name(name: &str) -> Result<(), ClockConfigurationError> {
    if name.is_empty() {
        return Err(ClockConfigurationError::EmptyName);
    }

    Ok(())
}
