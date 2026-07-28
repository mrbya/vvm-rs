extern crate vvm;

use std::convert::Infallible;
use vvm::{dut::Sample, timing::Clock};

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    enable: bool,

    #[vvm(port = "reset_n")]
    reset: bool,
}

#[derive(vvm_macros::Sample)]
#[vvm(dut = MockDut)]
struct Observation {
    #[vvm(port)]
    data_out: u8,

    #[vvm(port)]
    enable_out: bool,
}

#[derive(vvm_macros::Clock)]
#[vvm(dut = MockDut, clock = "clk")]
struct Clk;

struct MockDut {
    time: vvm::timing::SimulationTime,
}

impl vvm::dut::Dut for MockDut {
    type Error = Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn simulation_time(&self) -> vvm::timing::SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: vvm::timing::TimeStep) -> Result<(), Self::Error> {
        if let Some(time) = self.time.checked_add(delta) {
            self.time = time;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl MockDut {
    pub fn set_enable(
        &mut self,
        _value: impl ::core::borrow::Borrow<bool>,
    ) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn set_reset_n(
        &mut self,
        _value: impl ::core::borrow::Borrow<bool>,
    ) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn set_clk(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn data_out(&self) -> Result<u8, Infallible> {
        Ok(0)
    }

    pub fn enable_out(&self) -> Result<bool, Infallible> {
        Ok(false)
    }
}

fn main() -> Result<(), Infallible> {
    let stimulus = Stimulus {
        enable: true,
        reset: false,
    };
    let mut dut = MockDut {
        time: vvm::timing::SimulationTime::ZERO,
    };
    let mut clk = Clk;

    vvm::dut::Drive::drive(&stimulus, &mut dut)?;
    Observation::sample(&dut)?;
    clk.drive_inactive(&mut dut)?;
    clk.drive_active(&mut dut)?;

    Ok(())
}
