//! Packed benchmark support.

use std::time::Duration;

use criterion::{BenchmarkGroup, Throughput, measurement::WallTime};

/// Representative packed widths for deterministic benchmark coverage.
pub const REPRESENTATIVE_WIDTHS: [usize; 10] = [1, 8, 32, 63, 64, 65, 127, 128, 256, 1024];

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

/// Returns deterministic canonical words for one packed width.
pub fn words_for_width(width: usize) -> Vec<u32> {
    let word_count = width.div_ceil(32);
    let mut words = (0..word_count)
        .map(|index| {
            let ordinal = u32::try_from(index).unwrap_or_default();
            0x9e37_79b9_u32.wrapping_mul(ordinal.wrapping_add(1)) ^ 0xa5a5_5a5a
        })
        .collect::<Vec<_>>();

    if let Some(last) = words.last_mut() {
        let used_bits = width % 32;

        if used_bits != 0 {
            *last &= u32::MAX >> (32 - used_bits);
        }
    }

    words
}

/// Deterministic scalar fixture value truncated to the requested width.
pub fn scalar_value(width: usize) -> u64 {
    let value = 0x0123_4567_89ab_cdef_u64;

    if width == 64 {
        value
    } else {
        let shift = u32::try_from(width).unwrap_or_default();
        value & (u64::MAX >> (64 - shift))
    }
}
