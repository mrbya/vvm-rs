extern crate vvm;

use std::convert::Infallible;

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
    fn data_out(&mut self) -> Result<u8, Infallible> {
        Ok(0)
    }

    fn en_out(&mut self) -> Result<bool, Infallible> {
        Ok(false)
    }
}

#[derive(vvm_macros::Sample)]
#[vvm(dut = crate::Counter)]
struct Observation {
    #[vvm(port)]
    data_out: u8,

    en_out: bool,
}

fn main() {}
