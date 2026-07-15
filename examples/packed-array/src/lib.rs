//! Packed-array verification example with standard Rust test discovery.

#[cfg(test)]
vvm::include_dut!(packed_array_ports);

/// Packed-array VVM test cases.
#[cfg(test)]
mod test_cases;

/// Packed-array verification components.
#[cfg(test)]
mod verification;
