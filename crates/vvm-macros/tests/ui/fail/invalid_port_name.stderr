use std::convert::Infallible;

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
struct Stimulus {
    #[vvm(port = "invalid")]
    enable: bool,
}

fn main() {}
