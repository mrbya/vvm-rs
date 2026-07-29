//! Packed-struct verification example with standard Rust test discovery.

#[cfg(test)]
vvm::include_dut!(packed_struct_ports);

/// Packed-struct VVM test cases.
#[cfg(test)]
mod test_cases;

/// Packed-struct verification components.
#[cfg(test)]
mod verification;
