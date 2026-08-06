//! Scheduler benchmark support.

use std::num::NonZeroU64;
use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Throughput};
use vvm_core::{
    Clock, ClockScheduler, ClockTiming, CycleTiming, Drive, Dut, ExactScoreboard, ReferenceModel,
    Sample, SimulationTime, TestResult, Testbench, TimeStep, TimedDut, TimingScheduler,
};

/// Result type returned by deterministic mock testbench runs.
pub type MockRunResult = TestResult<
    MockStimulus,
    vvm_core::Mismatch<MockObservation, MockObservation>,
    std::convert::Infallible,
>;

/// Applies a smaller-sample configuration for expensive workloads.
pub fn configure_expensive_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));
}

/// Applies the default Criterion configuration for fast in-process benchmarks.
pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
}

/// Sets cycle throughput on one benchmark group.
pub fn throughput_cycles(group: &mut BenchmarkGroup<'_, WallTime>, cycles: u64) {
    group.throughput(Throughput::Elements(cycles));
}

/// Sets event throughput on one benchmark group.
pub fn throughput_events(group: &mut BenchmarkGroup<'_, WallTime>, events: u64) {
    group.throughput(Throughput::Elements(events));
}

/// Deterministic mock DUT used by testbench benchmarks.
#[derive(Debug, Clone, Default)]
pub struct MockDut {
    /// Driven clock state.
    pub clock: bool,
    /// Driven reset state.
    pub reset_n: bool,
    /// Driven enable state.
    pub enable: bool,
    /// Driven payload.
    pub payload: u32,
    /// Registered output value.
    pub observed: u32,
    /// Current simulation time.
    pub time: SimulationTime,
}

impl Dut for MockDut {
    type Error = std::convert::Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        if !self.reset_n {
            self.observed = 0;
        } else if self.clock && self.enable {
            self.observed = self.observed.wrapping_add(self.payload.wrapping_add(1));
        }

        Ok(())
    }

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        self.time = self.time.checked_add(delta).unwrap_or(self.time);
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Deterministic stimulus for one synchronous mock cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockStimulus {
    /// Active-low reset.
    pub reset_n: bool,
    /// Enable bit.
    pub enable: bool,
    /// Payload increment.
    pub payload: u32,
}

impl MockStimulus {
    /// Creates one deterministic sequence with a single reset cycle first.
    #[must_use]
    pub fn sequence(len: usize) -> Vec<Self> {
        let mut values = Vec::with_capacity(len);

        for index in 0..len {
            let payload = u32::try_from(index).unwrap_or_default() & 0x1f;
            values.push(Self {
                reset_n: index != 0,
                enable: index % 5 != 0,
                payload,
            });
        }

        values
    }
}

impl Drive<MockDut> for MockStimulus {
    fn drive(&self, dut: &mut MockDut) -> Result<(), <MockDut as Dut>::Error> {
        dut.reset_n = self.reset_n;
        dut.enable = self.enable;
        dut.payload = self.payload;
        Ok(())
    }
}

/// Deterministic observation sampled from the mock DUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockObservation {
    /// Registered output value.
    pub observed: u32,
}

impl Sample<MockDut> for MockObservation {
    fn sample(dut: &MockDut) -> Result<Self, <MockDut as Dut>::Error> {
        Ok(Self {
            observed: dut.observed,
        })
    }
}

/// Deterministic reference model for the mock DUT.
#[derive(Debug, Clone, Copy, Default)]
pub struct MockModel {
    /// Registered model output.
    observed: u32,
}

impl ReferenceModel<MockStimulus> for MockModel {
    type Expected = MockObservation;

    fn predict(&mut self, stimulus: &MockStimulus) -> Self::Expected {
        if !stimulus.reset_n {
            self.observed = 0;
        } else if stimulus.enable {
            self.observed = self.observed.wrapping_add(stimulus.payload.wrapping_add(1));
        }

        MockObservation {
            observed: self.observed,
        }
    }
}

/// Rising-edge clock driver for the mock DUT.
#[derive(Debug, Clone, Copy, Default)]
pub struct MockClock;

impl Clock<MockDut> for MockClock {
    fn drive_inactive(&mut self, dut: &mut MockDut) -> Result<(), <MockDut as Dut>::Error> {
        dut.clock = false;
        Ok(())
    }

    fn drive_active(&mut self, dut: &mut MockDut) -> Result<(), <MockDut as Dut>::Error> {
        dut.clock = true;
        Ok(())
    }
}

/// Executes one deterministic single-clock mock-DUT testbench run.
#[must_use]
pub fn run_mock_testbench(cycles: usize) -> MockRunResult {
    Testbench::new(MockDut::default())
        .with_sequence(MockStimulus::sequence(cycles))
        .with_reference_model(MockModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run::<MockObservation>()
}

/// Executes one deterministic multi-clock mock-DUT testbench run.
#[must_use]
pub fn run_multiclock_testbench(cycles: usize, clock_count: usize) -> MockRunResult {
    let builder = ClockScheduler::new("core", MockClock, ClockTiming::UNIT);
    let scheduler_result = match clock_count {
        1 => builder,
        2 => builder.and_then(|scheduler| {
            scheduler.with_clock(
                "peripheral",
                MockClock,
                ClockTiming::new(
                    CycleTiming::new(
                        TimeStep::new(1).unwrap_or(TimeStep::ONE),
                        TimeStep::new(3).unwrap_or(TimeStep::ONE),
                    ),
                    TimeStep::new(1).unwrap_or(TimeStep::ONE),
                ),
            )
        }),
        _ => builder
            .and_then(|scheduler| {
                scheduler.with_clock(
                    "peripheral",
                    MockClock,
                    ClockTiming::new(
                        CycleTiming::new(
                            TimeStep::new(1).unwrap_or(TimeStep::ONE),
                            TimeStep::new(3).unwrap_or(TimeStep::ONE),
                        ),
                        TimeStep::new(1).unwrap_or(TimeStep::ONE),
                    ),
                )
            })
            .and_then(|scheduler| {
                scheduler.with_clock(
                    "debug",
                    MockClock,
                    ClockTiming::new(
                        CycleTiming::new(
                            TimeStep::new(4).unwrap_or(TimeStep::ONE),
                            TimeStep::new(2).unwrap_or(TimeStep::ONE),
                        ),
                        TimeStep::new(2).unwrap_or(TimeStep::ONE),
                    ),
                )
            })
            .and_then(|scheduler| {
                scheduler.with_clock(
                    "io",
                    MockClock,
                    ClockTiming::new(
                        CycleTiming::new(
                            TimeStep::new(5).unwrap_or(TimeStep::ONE),
                            TimeStep::new(1).unwrap_or(TimeStep::ONE),
                        ),
                        TimeStep::new(3).unwrap_or(TimeStep::ONE),
                    ),
                )
            }),
    };

    let Ok(scheduler) = scheduler_result else {
        return Testbench::new(MockDut::default())
            .with_sequence(MockStimulus::sequence(cycles))
            .with_reference_model(MockModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(MockClock)
            .run::<MockObservation>();
    };

    Testbench::new(MockDut::default())
        .with_sequence(MockStimulus::sequence(cycles))
        .with_reference_model(MockModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clocks(scheduler)
        .run::<MockObservation>()
}

/// Deterministic delayed-event mock DUT.
#[derive(Debug, Clone)]
pub struct MockTimedDut {
    /// Current simulation time.
    pub time: SimulationTime,
    /// Remaining absolute event times.
    pub events: std::collections::VecDeque<SimulationTime>,
    /// Number of evaluations.
    pub evaluations: u64,
}

impl MockTimedDut {
    /// Creates one timed DUT from absolute event ticks.
    #[must_use]
    pub fn new(events: impl IntoIterator<Item = u64>) -> Self {
        Self {
            time: SimulationTime::ZERO,
            events: events.into_iter().map(SimulationTime::from_ticks).collect(),
            evaluations: 0,
        }
    }
}

impl Dut for MockTimedDut {
    type Error = std::convert::Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        self.evaluations = self.evaluations.saturating_add(1);

        if self.events.front() == Some(&self.time) {
            let _ = self.events.pop_front();
        }

        Ok(())
    }

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        self.time = self.time.checked_add(delta).unwrap_or(self.time);
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl TimedDut for MockTimedDut {
    fn events_pending(&self) -> Result<bool, Self::Error> {
        Ok(!self.events.is_empty())
    }

    fn next_time_slot(&self) -> Result<Option<SimulationTime>, Self::Error> {
        Ok(self.events.front().copied())
    }
}

/// Returns a deterministic dense timed workload.
#[must_use]
pub fn dense_event_ticks(count: usize) -> Vec<u64> {
    (1..=count)
        .map(|index| u64::try_from(index).unwrap_or_default())
        .collect()
}

/// Returns a deterministic sparse timed workload.
#[must_use]
pub fn sparse_event_ticks(count: usize) -> Vec<u64> {
    (1..=count)
        .map(|index| u64::try_from(index).unwrap_or_default().saturating_mul(10))
        .collect()
}

/// Runs one timed scheduler workload until the queue is empty.
pub fn run_timing_scheduler(events: &[u64]) {
    let mut dut = MockTimedDut::new(events.iter().copied());
    let mut scheduler = TimingScheduler::new();
    let limit =
        NonZeroU64::new(u64::try_from(events.len()).unwrap_or(1)).unwrap_or(NonZeroU64::MIN);
    let result = scheduler.run_until_idle(&mut dut, limit);

    assert!(
        result.is_ok(),
        "deterministic timed benchmark workload must succeed"
    );
}
