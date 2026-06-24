extern crate vvm_core as vvm;

use std::convert::Infallible;

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

impl MockDut {
    fn set_enable(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }

    fn set_reset_n(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }
}

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    enable: bool,

    #[vvm(port = "reset_n")]
    reset: bool,
}

fn main() {}
