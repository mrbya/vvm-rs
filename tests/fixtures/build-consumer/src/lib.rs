#[cfg(test)]
vvm::include_dut!(counter);

#[cfg(test)]
mod tests {
    use vvm::dut::Dut as _;

    use crate::counter::Counter;

    #[test]
    fn generated_counter_evaluates() -> Result<(), Box<dyn std::error::Error>> {
        let mut dut = Counter::new()?;

        dut.set_reset_n(false)?;
        dut.evaluate()?;

        assert_eq!(dut.count()?, 0);

        Ok(())
    }
}
