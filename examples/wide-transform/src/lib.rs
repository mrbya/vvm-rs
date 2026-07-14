//! Wide-port verification example with standard Rust test discovery.

#[cfg(test)]
vvm::include_dut!(wide_transform);

/// Wide-transform VVM test cases.
#[cfg(test)]
mod test_cases;

/// Wide-transform verification components.
#[cfg(test)]
mod verification;
