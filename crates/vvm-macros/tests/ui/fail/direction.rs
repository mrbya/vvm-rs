extern crate vvm_core as vvm;

use std::convert::Infallible;

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    count: u8,
}

struct MockDut;

impl vvm::Dut for MockDut {
    type Error = Infallible;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
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
