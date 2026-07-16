use std::num::NonZeroU64;

use super::{Clock, ClockTiming};
use crate::{Dut, TimeStep};

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

impl std::fmt::Display for ClockConfigurationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::EmptyName => formatter.write_str("clock name must not be empty"),
            Self::DuplicateName { ref name } => {
                write!(formatter, "clock name `{name}` is already registered")
            }
        }
    }
}

impl std::error::Error for ClockConfigurationError {}

/// Current semantic phase of one clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockPhase {
    /// The clock is in its configured inactive level.
    Inactive,
    /// The clock is in its configured active level.
    Active,
}

impl ClockPhase {
    /// Returns the following phase.
    const fn next(self) -> Self {
        match self {
            Self::Inactive => Self::Active,
            Self::Active => Self::Inactive,
        }
    }
}

/// A nonzero countdown to one clock transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ClockCountdown(NonZeroU64);

impl ClockCountdown {
    /// Creates a countdown from a valid time step.
    const fn from_time_step(step: TimeStep) -> Self {
        Self::from_ticks(step.ticks())
    }

    /// Returns the number of ticks remaining.
    const fn ticks(self) -> u64 {
        self.0.get()
    }

    /// Returns the countdown as a time step.
    const fn as_time_step(self) -> TimeStep {
        TimeStep::from_nonzero(self.0)
    }

    /// Subtracts an elapsed step, returning no value when the transition is due.
    const fn checked_sub(self, elapsed: TimeStep) -> Option<Self> {
        match self.ticks().checked_sub(elapsed.ticks()) {
            Some(ticks) => match NonZeroU64::new(ticks) {
                Some(ticks) => Some(Self(ticks)),
                None => None,
            },
            None => None,
        }
    }

    /// Creates a countdown from a nonzero tick count.
    const fn from_ticks(ticks: u64) -> Self {
        match NonZeroU64::new(ticks) {
            Some(ticks) => Self(ticks),
            None => Self(NonZeroU64::MIN),
        }
    }
}

/// One independently scheduled clock domain.
struct ClockDomain<'clock, D>
where
    D: Dut,
{
    /// Diagnostic name.
    name: String,
    /// Heterogeneous clock driver.
    driver: Box<dyn Clock<D> + 'clock>,
    /// Timing configuration.
    timing: ClockTiming,
    /// Current semantic phase.
    phase: ClockPhase,
    /// Time remaining until the next transition.
    remaining: ClockCountdown,
}

impl<'clock, D> ClockDomain<'clock, D>
where
    D: Dut,
{
    /// Constructs an inactive clock domain without driving the DUT.
    fn new<C>(name: String, driver: C, timing: ClockTiming) -> Self
    where
        C: Clock<D> + 'clock,
    {
        Self {
            name,
            driver: Box::new(driver),
            timing,
            phase: ClockPhase::Inactive,
            remaining: ClockCountdown::from_time_step(timing.first_active_after()),
        }
    }
}

/// Schedules independently timed primary and secondary clock domains.
pub struct ClockScheduler<'clock, D>
where
    D: Dut,
{
    /// Required primary clock.
    primary: ClockDomain<'clock, D>,
    /// Secondary clocks in registration order.
    secondary: Vec<ClockDomain<'clock, D>>,
}

impl<'clock, D> ClockScheduler<'clock, D>
where
    D: Dut,
{
    /// Creates the compatibility scheduler for one unnamed primary clock.
    pub(crate) fn single<C>(clock: C, timing: ClockTiming) -> Self
    where
        C: Clock<D> + 'clock,
    {
        Self {
            primary: ClockDomain::new("clock".to_owned(), clock, timing),
            secondary: Vec::new(),
        }
    }

    /// Creates a scheduler with one required primary clock.
    ///
    /// # Errors
    ///
    /// Returns [`ClockConfigurationError::EmptyName`] when the diagnostic name is empty.
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

    /// Returns the primary clock name.
    #[must_use]
    pub fn primary_name(&self) -> &str {
        &self.primary.name
    }

    /// Returns the total number of registered clocks.
    #[must_use]
    pub const fn clock_count(&self) -> usize {
        self.secondary.len().saturating_add(1)
    }

    /// Iterates over names in deterministic scheduling order.
    pub fn clock_names(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.primary.name.as_str())
            .chain(self.secondary.iter().map(|domain| domain.name.as_str()))
    }

    /// Returns the primary timing configuration.
    #[must_use]
    pub const fn primary_timing(&self) -> ClockTiming {
        self.primary.timing
    }

    /// Drives all clocks to their initial inactive level.
    pub(crate) fn drive_initial_inactive(
        &mut self,
        dut: &mut D,
    ) -> Result<(), ClockDriveFailure<D::Error>> {
        drive_domain_inactive(&mut self.primary, dut)?;
        for domain in &mut self.secondary {
            drive_domain_inactive(domain, dut)?;
        }
        Ok(())
    }

    /// Returns the delay until the next transition batch.
    pub(crate) fn next_transition_after(&self) -> TimeStep {
        let mut next = self.primary.remaining;
        for domain in &self.secondary {
            if domain.remaining < next {
                next = domain.remaining;
            }
        }
        next.as_time_step()
    }

    /// Returns the delay until the next primary-clock transition.
    pub(crate) const fn primary_transition_after(&self) -> TimeStep {
        self.primary.remaining.as_time_step()
    }

    /// Drives every transition due at the next event time.
    pub(crate) fn drive_next_batch(
        &mut self,
        dut: &mut D,
    ) -> Result<ClockEventBatch, ClockDriveFailure<D::Error>> {
        let elapsed = self.next_transition_after();
        let due = ClockCountdown::from_time_step(elapsed);
        let mut primary_transition = None;
        let mut transition_count = 0_usize;

        if self.primary.remaining == due {
            primary_transition = Some(drive_domain_transition(&mut self.primary, dut)?);
            transition_count = transition_count.saturating_add(1);
        } else {
            reduce_remaining(&mut self.primary, elapsed);
        }

        for domain in &mut self.secondary {
            if domain.remaining == due {
                drive_domain_transition(domain, dut)?;
                transition_count = transition_count.saturating_add(1);
            } else {
                reduce_remaining(domain, elapsed);
            }
        }

        Ok(ClockEventBatch {
            elapsed,
            primary_transition,
            transition_count,
        })
    }

    /// Returns the current primary clock semantic phase.
    pub(crate) const fn primary_phase(&self) -> ClockPhase {
        self.primary.phase
    }

    /// Replaces the primary timing before the scheduler begins running.
    pub(crate) const fn replace_primary_timing(&mut self, timing: ClockTiming) {
        self.primary.timing = timing;
        self.primary.phase = ClockPhase::Inactive;
        self.primary.remaining = ClockCountdown::from_time_step(timing.first_active_after());
    }

    /// Returns whether a diagnostic name is already registered.
    fn contains_name(&self, name: &str) -> bool {
        self.primary.name == name || self.secondary.iter().any(|domain| domain.name == name)
    }
}

/// Failure while driving one named clock transition.
#[derive(Debug)]
pub struct ClockDriveFailure<E> {
    /// Clock diagnostic name.
    name: String,
    /// Semantic phase that was being driven.
    phase: ClockPhase,
    /// Underlying DUT error.
    source: E,
}

impl<E> ClockDriveFailure<E> {
    /// Constructs a clock-drive failure.
    const fn new(name: String, phase: ClockPhase, source: E) -> Self {
        Self {
            name,
            phase,
            source,
        }
    }

    /// Returns the failed clock name.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for structured runtime diagnostics.")
    )]
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Returns the attempted phase.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for structured runtime diagnostics.")
    )]
    pub(crate) const fn phase(&self) -> ClockPhase {
        self.phase
    }

    /// Returns the underlying error.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for structured runtime diagnostics.")
    )]
    pub(crate) const fn source(&self) -> &E {
        &self.source
    }

    /// Returns the underlying error.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for structured runtime diagnostics.")
    )]
    pub(crate) fn into_source(self) -> E {
        self.source
    }

    /// Decomposes this failure.
    pub(crate) fn into_parts(self) -> (String, ClockPhase, E) {
        (self.name, self.phase, self.source)
    }
}

impl<E> std::fmt::Display for ClockDriveFailure<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to drive clock `{}` to {:?}",
            self.name, self.phase
        )
    }
}

/// All clock transitions occurring at one scheduler event time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockEventBatch {
    /// Time elapsed since the preceding event.
    elapsed: TimeStep,
    /// Primary transition at this event, when present.
    primary_transition: Option<ClockPhase>,
    /// Number of clocks transitioned.
    transition_count: usize,
}

impl ClockEventBatch {
    /// Returns time elapsed since the previous batch.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for event-loop integration.")
    )]
    pub(crate) const fn elapsed(self) -> TimeStep {
        self.elapsed
    }

    /// Returns the primary transition, if one occurred.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for event-loop integration.")
    )]
    pub(crate) const fn primary_transition(self) -> Option<ClockPhase> {
        self.primary_transition
    }

    /// Returns the number of transitions in this batch.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Reserved for event-loop integration.")
    )]
    pub(crate) const fn transition_count(self) -> usize {
        self.transition_count
    }

    /// Returns whether the primary entered its active phase.
    pub(crate) const fn primary_became_active(self) -> bool {
        matches!(self.primary_transition, Some(ClockPhase::Active))
    }

    /// Returns whether the primary entered its inactive phase.
    pub(crate) const fn primary_became_inactive(self) -> bool {
        matches!(self.primary_transition, Some(ClockPhase::Inactive))
    }
}

/// Validates a diagnostic clock name.
const fn validate_clock_name(name: &str) -> Result<(), ClockConfigurationError> {
    if name.is_empty() {
        return Err(ClockConfigurationError::EmptyName);
    }
    Ok(())
}

/// Drives a domain's inactive level without changing scheduler state.
fn drive_domain_inactive<D>(
    domain: &mut ClockDomain<'_, D>,
    dut: &mut D,
) -> Result<(), ClockDriveFailure<D::Error>>
where
    D: Dut,
{
    domain
        .driver
        .drive_inactive(dut)
        .map_err(|source| ClockDriveFailure::new(domain.name.clone(), ClockPhase::Inactive, source))
}

/// Reduces a countdown known to be strictly greater than `elapsed`.
const fn reduce_remaining<D>(domain: &mut ClockDomain<'_, D>, elapsed: TimeStep)
where
    D: Dut,
{
    if let Some(remaining) = domain.remaining.checked_sub(elapsed) {
        domain.remaining = remaining;
    }
}

/// Drives the next semantic phase and updates state only on success.
fn drive_domain_transition<D>(
    domain: &mut ClockDomain<'_, D>,
    dut: &mut D,
) -> Result<ClockPhase, ClockDriveFailure<D::Error>>
where
    D: Dut,
{
    let target = domain.phase.next();
    let result = match target {
        ClockPhase::Inactive => domain.driver.drive_inactive(dut),
        ClockPhase::Active => domain.driver.drive_active(dut),
    };
    result.map_err(|source| ClockDriveFailure::new(domain.name.clone(), target, source))?;

    domain.phase = target;
    domain.remaining = ClockCountdown::from_time_step(match target {
        ClockPhase::Inactive => domain.timing.inactive_phase(),
        ClockPhase::Active => domain.timing.active_phase(),
    });
    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::{ClockConfigurationError, ClockPhase, ClockScheduler};
    use crate::{Clock, ClockTiming, CycleTiming, Dut, InvalidTimeStep, SimulationTime, TimeStep};

    /// Events recorded by independent clock drivers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Event {
        /// Primary clock inactive drive.
        CoreInactive,
        /// Primary clock active drive.
        CoreActive,
        /// Peripheral clock inactive drive.
        PeripheralInactive,
        /// Peripheral clock active drive.
        PeripheralActive,
        /// Debug clock inactive drive.
        DebugInactive,
        /// Debug clock active drive.
        DebugActive,
    }

    /// Clock signal selected by one test driver.
    #[derive(Debug, Clone, Copy)]
    enum Signal {
        Core,
        Peripheral,
        Debug,
    }

    /// Failure generated by a test clock.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MockError {
        DriveFailed,
    }

    impl std::fmt::Display for MockError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("mock clock drive failed")
        }
    }

    impl std::error::Error for MockError {}

    /// Recording DUT proving scheduler operations do not invoke lifecycle methods.
    struct RecordingDut {
        core_clock: bool,
        peripheral_clock: bool,
        debug_clock: bool,
        log: Rc<RefCell<Vec<Event>>>,
        evaluations: u32,
        advances: u32,
        finalizations: u32,
    }

    impl RecordingDut {
        /// Creates a clean recording DUT.
        fn new(log: Rc<RefCell<Vec<Event>>>) -> Self {
            Self {
                core_clock: false,
                peripheral_clock: false,
                debug_clock: false,
                log,
                evaluations: 0,
                advances: 0,
                finalizations: 0,
            }
        }
    }

    impl Dut for RecordingDut {
        type Error = MockError;

        fn evaluate(&mut self) -> Result<(), Self::Error> {
            self.evaluations = self.evaluations.saturating_add(1);
            Ok(())
        }
        fn simulation_time(&self) -> SimulationTime {
            SimulationTime::ZERO
        }
        fn advance_time(&mut self, _: TimeStep) -> Result<(), Self::Error> {
            self.advances = self.advances.saturating_add(1);
            Ok(())
        }
        fn finalize(&mut self) -> Result<(), Self::Error> {
            self.finalizations = self.finalizations.saturating_add(1);
            Ok(())
        }
    }

    /// Configurable clock driver used by scheduler tests.
    struct RecordingClock {
        signal: Signal,
        fails_on: Option<ClockPhase>,
    }

    impl RecordingClock {
        /// Creates a driver that never fails.
        const fn new(signal: Signal) -> Self {
            Self {
                signal,
                fails_on: None,
            }
        }

        /// Creates a driver that fails on a selected semantic phase.
        const fn failing(signal: Signal, phase: ClockPhase) -> Self {
            Self {
                signal,
                fails_on: Some(phase),
            }
        }

        /// Records one signal level transition.
        fn drive(&self, dut: &mut RecordingDut, phase: ClockPhase) -> Result<(), MockError> {
            if self.fails_on == Some(phase) {
                return Err(MockError::DriveFailed);
            }
            let event = match (self.signal, phase) {
                (Signal::Core, ClockPhase::Inactive) => {
                    dut.core_clock = false;
                    Event::CoreInactive
                }
                (Signal::Core, ClockPhase::Active) => {
                    dut.core_clock = true;
                    Event::CoreActive
                }
                (Signal::Peripheral, ClockPhase::Inactive) => {
                    dut.peripheral_clock = false;
                    Event::PeripheralInactive
                }
                (Signal::Peripheral, ClockPhase::Active) => {
                    dut.peripheral_clock = true;
                    Event::PeripheralActive
                }
                (Signal::Debug, ClockPhase::Inactive) => {
                    dut.debug_clock = false;
                    Event::DebugInactive
                }
                (Signal::Debug, ClockPhase::Active) => {
                    dut.debug_clock = true;
                    Event::DebugActive
                }
            };
            dut.log.borrow_mut().push(event);
            Ok(())
        }
    }

    impl Clock<RecordingDut> for RecordingClock {
        fn drive_inactive(&mut self, dut: &mut RecordingDut) -> Result<(), MockError> {
            self.drive(dut, ClockPhase::Inactive)
        }
        fn drive_active(&mut self, dut: &mut RecordingDut) -> Result<(), MockError> {
            self.drive(dut, ClockPhase::Active)
        }
    }

    /// Borrowing driver used to verify the scheduler lifetime is not static.
    struct BorrowedClock<'a> {
        drives: &'a Cell<u32>,
    }

    impl Clock<RecordingDut> for BorrowedClock<'_> {
        fn drive_inactive(&mut self, _: &mut RecordingDut) -> Result<(), MockError> {
            self.drives.set(self.drives.get().saturating_add(1));
            Ok(())
        }
        fn drive_active(&mut self, _: &mut RecordingDut) -> Result<(), MockError> {
            self.drives.set(self.drives.get().saturating_add(1));
            Ok(())
        }
    }

    fn timing(inactive: u64, active: u64, first: u64) -> Result<ClockTiming, InvalidTimeStep> {
        Ok(ClockTiming::new(
            CycleTiming::new(TimeStep::new(inactive)?, TimeStep::new(active)?),
            TimeStep::new(first)?,
        ))
    }

    fn scheduler(
        timing: ClockTiming,
    ) -> Result<ClockScheduler<'static, RecordingDut>, ClockConfigurationError> {
        ClockScheduler::new("core", RecordingClock::new(Signal::Core), timing)
    }

    fn drive_next(
        scheduler: &mut ClockScheduler<'_, RecordingDut>,
        dut: &mut RecordingDut,
    ) -> Result<super::ClockEventBatch, String> {
        scheduler
            .drive_next_batch(dut)
            .map_err(|failure| format!("{failure:?}"))
    }

    fn drive_initial(
        scheduler: &mut ClockScheduler<'_, RecordingDut>,
        dut: &mut RecordingDut,
    ) -> Result<(), String> {
        scheduler
            .drive_initial_inactive(dut)
            .map_err(|failure| format!("{failure:?}"))
    }

    #[test]
    fn rejects_empty_primary_name() {
        assert!(matches!(
            ClockScheduler::new("", RecordingClock::new(Signal::Core), ClockTiming::UNIT),
            Err(ClockConfigurationError::EmptyName)
        ));
    }

    #[test]
    fn rejects_empty_secondary_name() -> Result<(), ClockConfigurationError> {
        let mut scheduler = scheduler(ClockTiming::UNIT)?;
        assert_eq!(
            scheduler.add_clock(
                "",
                RecordingClock::new(Signal::Peripheral),
                ClockTiming::UNIT
            ),
            Err(ClockConfigurationError::EmptyName)
        );
        Ok(())
    }

    #[test]
    fn rejects_duplicate_clock_names() -> Result<(), ClockConfigurationError> {
        let mut scheduler = scheduler(ClockTiming::UNIT)?;
        assert_eq!(
            scheduler.add_clock(
                "core",
                RecordingClock::new(Signal::Peripheral),
                ClockTiming::UNIT
            ),
            Err(ClockConfigurationError::DuplicateName {
                name: "core".to_owned()
            })
        );
        scheduler.add_clock(
            "peripheral",
            RecordingClock::new(Signal::Peripheral),
            ClockTiming::UNIT,
        )?;
        assert_eq!(
            scheduler.add_clock(
                "peripheral",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT
            ),
            Err(ClockConfigurationError::DuplicateName {
                name: "peripheral".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn preserves_names_and_registration_order() -> Result<(), ClockConfigurationError> {
        let scheduler = scheduler(ClockTiming::UNIT)?
            .with_clock(
                " peripheral",
                RecordingClock::new(Signal::Peripheral),
                ClockTiming::UNIT,
            )?
            .with_clock(
                "CORE",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT,
            )?;
        assert_eq!(scheduler.primary_name(), "core");
        assert_eq!(scheduler.clock_count(), 3);
        assert_eq!(
            scheduler.clock_names().collect::<Vec<_>>(),
            vec!["core", " peripheral", "CORE"]
        );
        Ok(())
    }

    #[test]
    fn supports_borrowed_clock_drivers() -> Result<(), Box<dyn std::error::Error>> {
        let drives = Cell::new(0);
        let mut scheduler =
            ClockScheduler::new("core", BorrowedClock { drives: &drives }, ClockTiming::UNIT)?;
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(log);
        drive_initial(&mut scheduler, &mut dut)?;
        assert_eq!(drives.get(), 1);
        Ok(())
    }

    #[test]
    fn drives_initial_inactive_levels_in_registration_order(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(Rc::clone(&log));
        let mut scheduler = scheduler(ClockTiming::UNIT)?
            .with_clock(
                "peripheral",
                RecordingClock::new(Signal::Peripheral),
                ClockTiming::UNIT,
            )?
            .with_clock(
                "debug",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT,
            )?;
        drive_initial(&mut scheduler, &mut dut)?;
        assert_eq!(
            *log.borrow(),
            vec![
                Event::CoreInactive,
                Event::PeripheralInactive,
                Event::DebugInactive
            ]
        );
        Ok(())
    }

    #[test]
    fn schedules_independent_periods() -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(Rc::clone(&log));
        let mut scheduler = scheduler(timing(3, 1, 4)?)?.with_clock(
            "peripheral",
            RecordingClock::new(Signal::Peripheral),
            timing(1, 1, 1)?,
        )?;
        let expected = [
            (1, 1, None),
            (1, 1, None),
            (1, 1, None),
            (1, 2, Some(ClockPhase::Active)),
            (1, 2, Some(ClockPhase::Inactive)),
            (1, 1, None),
            (1, 1, None),
            (1, 2, Some(ClockPhase::Active)),
        ];
        let mut elapsed = 0_u64;
        for (delta, count, primary) in expected {
            let batch = drive_next(&mut scheduler, &mut dut)?;
            elapsed = elapsed.saturating_add(batch.elapsed().ticks());
            assert_eq!(batch.elapsed().ticks(), delta);
            assert_eq!(batch.transition_count(), count);
            assert_eq!(batch.primary_transition(), primary);
        }
        assert_eq!(elapsed, 8);
        assert_eq!(
            *log.borrow(),
            vec![
                Event::PeripheralActive,
                Event::PeripheralInactive,
                Event::PeripheralActive,
                Event::CoreActive,
                Event::PeripheralInactive,
                Event::CoreInactive,
                Event::PeripheralActive,
                Event::PeripheralInactive,
                Event::PeripheralActive,
                Event::CoreActive,
                Event::PeripheralInactive
            ]
        );
        Ok(())
    }

    #[test]
    fn custom_first_delay_only_applies_once() -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(log);
        let mut scheduler = scheduler(timing(3, 2, 7)?)?;
        for (delta, phase) in [
            (7, ClockPhase::Active),
            (2, ClockPhase::Inactive),
            (3, ClockPhase::Active),
            (2, ClockPhase::Inactive),
            (3, ClockPhase::Active),
        ] {
            let batch = drive_next(&mut scheduler, &mut dut)?;
            assert_eq!(batch.elapsed().ticks(), delta);
            assert_eq!(batch.primary_transition(), Some(phase));
        }
        Ok(())
    }

    #[test]
    fn batches_simultaneous_transitions_in_registration_order(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(Rc::clone(&log));
        let mut scheduler = scheduler(ClockTiming::UNIT)?
            .with_clock(
                "peripheral",
                RecordingClock::new(Signal::Peripheral),
                ClockTiming::UNIT,
            )?
            .with_clock(
                "debug",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT,
            )?;
        let first = drive_next(&mut scheduler, &mut dut)?;
        assert_eq!(first.transition_count(), 3);
        assert!(first.primary_became_active());
        assert_eq!(
            *log.borrow(),
            vec![
                Event::CoreActive,
                Event::PeripheralActive,
                Event::DebugActive
            ]
        );
        log.borrow_mut().clear();
        let second = drive_next(&mut scheduler, &mut dut)?;
        assert!(second.primary_became_inactive());
        assert_eq!(
            *log.borrow(),
            vec![
                Event::CoreInactive,
                Event::PeripheralInactive,
                Event::DebugInactive
            ]
        );
        Ok(())
    }

    #[test]
    fn preserves_state_and_stops_batch_after_secondary_failure(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(Rc::clone(&log));
        let mut scheduler = scheduler(ClockTiming::UNIT)?
            .with_clock(
                "peripheral",
                RecordingClock::failing(Signal::Peripheral, ClockPhase::Active),
                ClockTiming::UNIT,
            )?
            .with_clock(
                "debug",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT,
            )?;
        let Err(failure) = scheduler.drive_next_batch(&mut dut) else {
            return Err("expected clock failure".into());
        };
        assert_eq!(failure.name(), "peripheral");
        assert_eq!(failure.phase(), ClockPhase::Active);
        assert_eq!(*failure.source(), MockError::DriveFailed);
        let Some(failed_domain) = scheduler.secondary.first() else {
            return Err("missing failed secondary clock".into());
        };
        assert_eq!(failed_domain.phase, ClockPhase::Inactive);
        assert_eq!(failed_domain.remaining.as_time_step(), TimeStep::ONE);
        assert_eq!(*log.borrow(), vec![Event::CoreActive]);
        Ok(())
    }

    #[test]
    fn exposes_clock_failure_ownership_helpers() {
        let inactive_failure = super::ClockDriveFailure::new(
            "core".to_owned(),
            ClockPhase::Active,
            MockError::DriveFailed,
        );
        assert_eq!(inactive_failure.into_source(), MockError::DriveFailed);

        let parts_failure = super::ClockDriveFailure::new(
            "core".to_owned(),
            ClockPhase::Inactive,
            MockError::DriveFailed,
        );
        assert_eq!(
            parts_failure.into_parts(),
            (
                "core".to_owned(),
                ClockPhase::Inactive,
                MockError::DriveFailed
            )
        );
    }

    #[test]
    fn replaces_primary_timing_before_running() -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(log);
        let mut scheduler = scheduler(ClockTiming::UNIT)?;
        let replacement = timing(3, 2, 7)?;
        scheduler.replace_primary_timing(replacement);
        assert_eq!(scheduler.primary_timing(), replacement);
        assert_eq!(scheduler.primary_phase(), ClockPhase::Inactive);
        assert_eq!(scheduler.next_transition_after().ticks(), 7);
        let batch = drive_next(&mut scheduler, &mut dut)?;
        assert_eq!(batch.primary_transition(), Some(ClockPhase::Active));
        Ok(())
    }

    #[test]
    fn initialization_and_primary_failures_stop_later_drives(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(Rc::clone(&log));
        let mut initialization = scheduler(ClockTiming::UNIT)?
            .with_clock(
                "peripheral",
                RecordingClock::failing(Signal::Peripheral, ClockPhase::Inactive),
                ClockTiming::UNIT,
            )?
            .with_clock(
                "debug",
                RecordingClock::new(Signal::Debug),
                ClockTiming::UNIT,
            )?;
        let Err(failure) = initialization.drive_initial_inactive(&mut dut) else {
            return Err("expected initialization failure".into());
        };
        assert_eq!(failure.name(), "peripheral");
        assert_eq!(failure.phase(), ClockPhase::Inactive);
        assert_eq!(*log.borrow(), vec![Event::CoreInactive]);
        log.borrow_mut().clear();
        let mut primary = ClockScheduler::new(
            "core",
            RecordingClock::failing(Signal::Core, ClockPhase::Active),
            ClockTiming::UNIT,
        )?
        .with_clock(
            "peripheral",
            RecordingClock::new(Signal::Peripheral),
            ClockTiming::UNIT,
        )?;
        let Err(primary_failure) = primary.drive_next_batch(&mut dut) else {
            return Err("expected primary failure".into());
        };
        assert_eq!(primary_failure.name(), "core");
        assert_eq!(primary_failure.phase(), ClockPhase::Active);
        assert_eq!(primary.primary_phase(), ClockPhase::Inactive);
        assert!(log.borrow().is_empty());
        Ok(())
    }

    #[test]
    fn scheduler_never_runs_dut_lifecycle_operations() -> Result<(), Box<dyn std::error::Error>> {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut dut = RecordingDut::new(log);
        let mut scheduler = scheduler(ClockTiming::UNIT)?;
        drive_initial(&mut scheduler, &mut dut)?;
        drive_next(&mut scheduler, &mut dut)?;
        assert_eq!(dut.evaluations, 0);
        assert_eq!(dut.advances, 0);
        assert_eq!(dut.finalizations, 0);
        Ok(())
    }
}
