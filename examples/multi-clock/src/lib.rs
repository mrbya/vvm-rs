//! Independently timed multi-clock verification example.

#[cfg(test)]
vvm::include_dut!(multi_clock_counter);

#[cfg(test)]
mod test_cases;
#[cfg(test)]
mod verification;
