extern crate vvm;

use std::convert::Infallible;

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    count: u8,
}

struct MockDut { time: vvm::timing::SimulationTime }

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
    fn count(&self) -> Result<u8, Infallible> {
        Ok(0)
    }
}

fn main() {}
