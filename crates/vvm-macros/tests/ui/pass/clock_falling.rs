extern crate vvm;

use std::convert::Infallible;

#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk",
    edge = "falling"
)]
struct MockClock;

struct MockDut {
    clk: bool,
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
    fn set_clk(&mut self, value: bool) -> Result<(), Infallible> {
        self.clk = value;
        Ok(())
    }
}

fn main() -> Result<(), Infallible> {
    let mut clock = MockClock;
    let mut dut = MockDut {
        clk: false,
        time: vvm::timing::SimulationTime::ZERO,
    };

    vvm::timing::Clock::drive_inactive(&mut clock, &mut dut)?;

    assert!(dut.clk);

    vvm::timing::Clock::drive_active(&mut clock, &mut dut)?;

    assert!(!dut.clk);

    Ok(())
}
