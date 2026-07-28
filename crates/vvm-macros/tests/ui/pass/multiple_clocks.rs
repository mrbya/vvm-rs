extern crate vvm;

use vvm::{dut::Dut, timing::{Clock, SimulationTime, TimeStep}};

struct MockDut {
    core_clk: bool,
    peripheral_clk: bool,
}

#[derive(Debug)]
struct MockError;

impl MockDut {
    fn set_core_clk(&mut self, value: bool) -> Result<(), MockError> {
        self.core_clk = value;
        Ok(())
    }

    fn set_peripheral_clk(&mut self, value: bool) -> Result<(), MockError> {
        self.peripheral_clk = value;
        Ok(())
    }
}

impl Dut for MockDut {
    type Error = MockError;

    fn evaluate(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn simulation_time(&self) -> SimulationTime { SimulationTime::ZERO }
    fn advance_time(&mut self, _: TimeStep) -> Result<(), Self::Error> { Ok(()) }
    fn finalize(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

#[derive(vvm_macros::Clock)]
#[vvm(dut = MockDut, clock = "core_clk")]
struct CoreClock;

#[derive(vvm_macros::Clock)]
#[vvm(dut = MockDut, clock = "peripheral_clk", edge = "falling")]
struct PeripheralClock;

fn assert_clock<C: Clock<MockDut>>() {}

fn main() {
    assert_clock::<CoreClock>();
    assert_clock::<PeripheralClock>();
}
