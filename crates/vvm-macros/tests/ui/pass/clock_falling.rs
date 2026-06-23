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
}

impl vvm_core::Dut for MockDut {
    type Error = Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
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
    let mut dut = MockDut { clk: false };

    vvm_core::Clock::drive_inactive(&mut clock, &mut dut)?;

    assert!(dut.clk);

    vvm_core::Clock::drive_active(&mut clock, &mut dut)?;

    assert!(!dut.clk);

    Ok(())
}
