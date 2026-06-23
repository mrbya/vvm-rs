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
