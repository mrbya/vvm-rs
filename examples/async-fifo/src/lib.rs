//! Asynchronous FIFO verification with independent clock domains.

#[cfg(test)]
vvm::include_dut!(async_fifo);

#[cfg(test)]
/// Directed multi-domain FIFO verification scenarios.
mod test_cases;
#[cfg(test)]
/// Direct generated-DUT operations shared by asynchronous FIFO scenarios.
mod verification;
