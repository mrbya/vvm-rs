use std::convert::Infallible;
use vvm_core::{Clock, Sample};

#[derive(vvm_macros::Drive)]
#[vvm(dut = MockDut)]
struct Stimulus {
    #[vvm(port)]
    enable: bool,

    #[vvm(port = "reset_n")]
    reset: bool,
}

#[derive(vvm_macros::Sample)]
#[vvm(dut = MockDut)]
struct Observation {
    #[vvm(port)]
    data_out: u8,

    #[vvm(port)]
    enable_out: bool,
}

#[derive(vvm_macros::Clock)]
#[vvm(dut = MockDut, clock = "clk")]
struct Clk;

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
    pub fn set_enable(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn set_reset_n(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn set_clk(&mut self, _value: bool) -> Result<(), Infallible> {
        Ok(())
    }

    pub fn data_out(&self) -> Result<u8, Infallible> {
        Ok(0)
    }

    pub fn enable_out(&self) -> Result<bool, Infallible> {
        Ok(false)
    }
}

fn main() -> Result<(), Infallible> {
    let stimulus = Stimulus {
        enable: true,
        reset: false,
    };
    let mut dut = MockDut;
    let mut clk = Clk;

    vvm_core::Drive::drive(&stimulus, &mut dut)?;
    Observation::sample(&dut)?;
    clk.drive_inactive(&mut dut)?;
    clk.drive_active(&mut dut)?;

    Ok(())
}
