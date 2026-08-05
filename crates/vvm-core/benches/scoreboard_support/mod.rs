//! Scoreboard benchmark support.

use std::time::Duration;

use criterion::{BenchmarkGroup, Throughput, measurement::WallTime};

/// Applies the default Criterion configuration for fast in-process benchmarks.
pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
}

/// Sets element throughput on one benchmark group.
pub fn throughput_elements(group: &mut BenchmarkGroup<'_, WallTime>, elements: u64) {
    group.throughput(Throughput::Elements(elements));
}

/// One structured scoreboard payload with stable data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredValue {
    /// Stable scalar header.
    header: u32,
    /// Stable scalar payload.
    payload: u64,
    /// Stable repeated bytes.
    bytes: [u8; 64],
}

impl StructuredValue {
    /// Creates a deterministic structured value.
    #[must_use]
    pub fn new(fill: u8) -> Self {
        Self {
            header: 0xfeed_cafe,
            payload: 0x0123_4567_89ab_cdef,
            bytes: [fill; 64],
        }
    }

    /// Creates a copy with one early mismatch.
    #[must_use]
    pub fn early_mismatch(mut self) -> Self {
        self.bytes[0] ^= 1;
        self
    }

    /// Creates a copy with one late mismatch.
    #[must_use]
    pub fn late_mismatch(mut self) -> Self {
        self.bytes[63] ^= 1;
        self
    }
}
