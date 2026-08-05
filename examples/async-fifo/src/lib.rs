//! Asynchronous FIFO verification with independent clock domains.

vvm::include_dut!(async_fifo);

#[doc(hidden)]
pub mod benchmark;

#[cfg(test)]
/// Directed multi-domain FIFO verification scenarios.
mod test_cases;
/// Direct generated-DUT operations shared by asynchronous FIFO scenarios.
mod verification;
