use thiserror::Error;
use vvm::testbench::{Mismatch, ReferenceModel, TestResult};
use vvm::timing::{
    ClockConfigurationError, ClockScheduler, ClockTiming, CycleTiming, InvalidTimeStep, TimeStep,
};
use vvm::{Clock, Drive, Sample};

use crate::multi_clock_counter::{MultiClockCounter, MultiClockCounterError};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Dut(#[from] MultiClockCounterError),
    #[error(transparent)]
    InvalidTimeStep(#[from] InvalidTimeStep),
    #[error(transparent)]
    ClockConfiguration(#[from] ClockConfigurationError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub type MultiClockMismatch = Mismatch<MultiClockObservation, MultiClockObservation>;
pub type MultiClockTestResult =
    TestResult<MultiClockStimulus, MultiClockMismatch, MultiClockCounterError>;

#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(dut = MultiClockCounter, clock = "core_clk")]
pub struct CoreClock;

#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(dut = MultiClockCounter, clock = "peripheral_clk")]
pub struct PeripheralClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Drive)]
#[vvm(dut = MultiClockCounter)]
pub struct MultiClockStimulus {
    #[vvm(port)]
    reset_n: bool,
    #[vvm(port)]
    peripheral_enable: bool,
}

impl MultiClockStimulus {
    pub const fn new(reset_n: bool, peripheral_enable: bool) -> Self {
        Self {
            reset_n,
            peripheral_enable,
        }
    }
    pub const fn reset_n(self) -> bool {
        self.reset_n
    }
    pub const fn peripheral_enable(self) -> bool {
        self.peripheral_enable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sample)]
#[vvm(dut = MultiClockCounter)]
pub struct MultiClockObservation {
    #[vvm(port)]
    peripheral_count: u8,
    #[vvm(port)]
    core_sample: u8,
}

impl MultiClockObservation {
    pub const fn new(peripheral_count: u8, core_sample: u8) -> Self {
        Self {
            peripheral_count,
            core_sample,
        }
    }
}

pub fn clock_scheduler() -> Result<ClockScheduler<'static, MultiClockCounter>> {
    let core = ClockTiming::new(
        CycleTiming::new(TimeStep::new(3)?, TimeStep::ONE),
        TimeStep::new(4)?,
    );
    Ok(ClockScheduler::new("core", CoreClock, core)?.with_clock(
        "peripheral",
        PeripheralClock,
        ClockTiming::from_cycle(CycleTiming::UNIT),
    )?)
}

#[derive(Debug, Default)]
pub struct MultiClockReferenceModel {
    peripheral_count: u8,
}

impl ReferenceModel<MultiClockStimulus> for MultiClockReferenceModel {
    type Expected = MultiClockObservation;
    fn predict(&mut self, stimulus: &MultiClockStimulus) -> Self::Expected {
        if !stimulus.reset_n() {
            self.peripheral_count = 0;
        } else if stimulus.peripheral_enable() {
            self.peripheral_count = self.peripheral_count.wrapping_add(2);
        }
        MultiClockObservation::new(self.peripheral_count, self.peripheral_count)
    }
}

pub fn multi_clock_sequence() -> impl ExactSizeIterator<Item = MultiClockStimulus> {
    [
        MultiClockStimulus::new(false, false),
        MultiClockStimulus::new(true, false),
        MultiClockStimulus::new(true, true),
        MultiClockStimulus::new(true, true),
        MultiClockStimulus::new(true, false),
        MultiClockStimulus::new(true, true),
        MultiClockStimulus::new(false, true),
        MultiClockStimulus::new(true, true),
    ]
    .into_iter()
}
