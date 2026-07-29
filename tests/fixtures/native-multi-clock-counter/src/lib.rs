//! Focused regression for generated multiple-clock DUT handling.

#[cfg(test)]
vvm::include_dut!(multi_clock_counter);

#[cfg(test)]
mod tests {
    use vvm::dut::Dut;
    use vvm::timing::TimeStep;

    use crate::multi_clock_counter::MultiClockCounter;

    #[test]
    fn independently_clocked_counter_preserves_edge_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut dut = MultiClockCounter::new()?;
        dut.set_core_clk(false)?;
        dut.set_peripheral_clk(false)?;
        dut.set_reset_n(true)?;
        dut.eval()?;
        dut.set_reset_n(false)?;
        dut.eval()?;
        dut.set_reset_n(true)?;
        dut.set_peripheral_enable(true)?;

        dut.set_peripheral_clk(true)?;
        dut.eval()?;
        dut.set_peripheral_clk(false)?;
        dut.eval()?;
        Dut::advance_time(&mut dut, TimeStep::ONE)?;

        dut.set_core_clk(true)?;
        dut.eval()?;
        dut.set_core_clk(false)?;
        dut.eval()?;

        assert_eq!(dut.peripheral_count()?, 1);
        assert_eq!(dut.core_sample()?, 1);
        dut.finish()?;
        Ok(())
    }
}
