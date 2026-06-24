extern crate vvm_core as vvm;

use std::convert::Infallible;

#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk"
)]
struct MockClock;

struct MockDut { time: vvm::SimulationTime }

impl vvm::Dut for MockDut {
    type Error = Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn simulation_time(&self) -> vvm::SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: vvm::TimeStep) -> Result<(), Self::Error> {
        if let Some(time) = self.time.checked_add(delta) {
            self.time = time;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {}
