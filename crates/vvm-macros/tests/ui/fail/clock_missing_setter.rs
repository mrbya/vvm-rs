use std::convert::Infallible;

#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk"
)]
struct MockClock;

struct MockDut;

impl vvm_core::Dut for MockDut {
    type Error = Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {}
