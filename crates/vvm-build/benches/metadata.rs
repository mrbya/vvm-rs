//! Criterion benchmarks for `vvm-build` metadata and code generation.

mod metadata_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use metadata_support::{FIXTURE_NAMES, configure_group, throughput_fixtures};
use tempfile::tempdir;
use vvm_build::benchmark::FixtureBench;

/// Benchmarks raw metadata decoding from committed JSON fixtures.
fn metadata_decode_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vvm-build/metadata/decode");
    configure_group(&mut group);

    throughput_fixtures(&mut group, u64::try_from(FIXTURE_NAMES.len()).unwrap_or_default());

    for fixture in FIXTURE_NAMES {
        group.bench_with_input(BenchmarkId::from_parameter(fixture), &fixture, |bench, name| {
            bench.iter(|| black_box(FixtureBench::decode(name)));
        });
    }

    group.finish();
}

/// Benchmarks normalization and supported-feature validation.
fn metadata_normalize_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vvm-build/metadata/normalize");
    configure_group(&mut group);

    throughput_fixtures(&mut group, u64::try_from(FIXTURE_NAMES.len()).unwrap_or_default());

    for fixture in FIXTURE_NAMES {
        group.bench_with_input(BenchmarkId::from_parameter(fixture), &fixture, |bench, name| {
            bench.iter(|| {
                black_box((
                    FixtureBench::normalize(name),
                    FixtureBench::validate_supported(name),
                ))
            });
        });
    }

    group.finish();
}

/// Benchmarks type mapping and complete generated source construction.
fn codegen_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vvm-build/codegen");
    configure_group(&mut group);

    for fixture in FIXTURE_NAMES {
        group.bench_with_input(BenchmarkId::new("types", fixture), &fixture, |bench, name| {
            bench.iter(|| black_box(FixtureBench::map_types(name)));
        });

        group.bench_with_input(BenchmarkId::new("names", fixture), &fixture, |bench, name| {
            bench.iter(|| black_box(FixtureBench::resolve_names(name)));
        });

        group.bench_with_input(BenchmarkId::new("generate", fixture), &fixture, |bench, name| {
            bench.iter(|| {
                let output = tempdir().unwrap_or_else(|_| unreachable!());
                black_box(FixtureBench::generate(name, output.path()))
            });
        });
    }

    group.finish();
}

criterion_group!(metadata_benches, metadata_decode_benches, metadata_normalize_benches, codegen_benches);
criterion_main!(metadata_benches);
