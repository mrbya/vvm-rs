//! Deterministic support for `vvm-build` metadata benchmarks.

use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Throughput};

/// Representative committed metadata fixture names.
pub const FIXTURE_NAMES: [&str; 9] = [
    "counter",
    "wide_ports",
    "signed_ports",
    "packed_array_ports",
    "packed_struct_ports",
    "packed_enum_ports",
    "unpacked_array_ports",
    "inout_ports",
    "aggregate_ports",
];

/// Applies the default Criterion configuration for metadata benchmarks.
pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(40);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
}

/// Sets fixture throughput on one benchmark group.
pub fn throughput_fixtures(group: &mut BenchmarkGroup<'_, WallTime>, fixtures: u64) {
    group.throughput(Throughput::Elements(fixtures));
}
