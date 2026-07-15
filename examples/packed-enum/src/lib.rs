//! Packed-enum verification example with standard Rust test discovery.

#[cfg(test)]
vvm::include_dut!(packed_enum_ports);

/// Packed-enum VVM test cases.
#[cfg(test)]
mod test_cases;

/// Packed-enum verification components.
#[cfg(test)]
mod verification;
