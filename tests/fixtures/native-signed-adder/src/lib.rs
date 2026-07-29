//! Signed-adder verification example with standard Rust test discovery.

#[cfg(test)]
vvm::include_dut!(signed_adder);

/// Signed-adder VVM test cases.
#[cfg(test)]
mod test_cases;

/// Signed-adder verification components.
#[cfg(test)]
mod verification;
