extern crate vvm;

use std::borrow::Borrow;
use std::convert::Infallible;

/// Deliberately neither `Copy` nor `Clone`.
struct NonCopyValue {
    value: u32,
}

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    payload: NonCopyValue,
}

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
    pub fn set_payload(&mut self, value: impl Borrow<NonCopyValue>) -> Result<(), Infallible> {
        let _payload = value.borrow().value;

        Ok(())
    }
}

fn main() -> Result<(), Infallible> {
    let stimulus = Stimulus {
        payload: NonCopyValue { value: 0x1234_5678 },
    };

    let mut dut = MockDut {
        time: vvm::timing::SimulationTime::ZERO,
    };

    vvm::dut::Drive::drive(&stimulus, &mut dut)?;

    Ok(())
}
