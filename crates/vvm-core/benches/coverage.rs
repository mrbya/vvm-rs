//! Criterion benchmarks for coverage sampling, persistence, merging, and reporting.

mod coverage_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use coverage_support::{
    CoverageFixture, artifact_with_samples, configure_group, merge_fixture, throughput_artifacts,
    throughput_samples,
};
use vvm_core::{CoverageArtifact, CoverageMerge, CoverageMergePolicy, TestStatus};

/// Benchmarks coverage sampling and snapshot capture.
fn coverage_sampling_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("coverage/sampling");
    configure_group(&mut group);

    group.bench_function("coverpoint-and-cross", |bench| {
        bench.iter(|| {
            let mut fixture = CoverageFixture::new("dut.coverage").unwrap_or_else(|_| unreachable!());
            black_box(fixture.sample_pair(4, 2))
        });
    });

    group.finish();
}

/// Benchmarks artifact encoding and decoding.
fn artifact_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("coverage/artifact");
    configure_group(&mut group);

    let pairs = [(0_u8, 0_u8), (1, 1), (4, 2), (9, 1), (3, 2), (8, 0)];
    let artifact = artifact_with_samples("artifact_bench", TestStatus::Passed, &pairs)
        .unwrap_or_else(|_| unreachable!());
    let json = artifact.to_json().unwrap_or_default();

    throughput_samples(&mut group, u64::try_from(pairs.len()).unwrap_or_default());

    group.bench_function("encode", |bench| {
        bench.iter(|| black_box(artifact.to_json()));
    });

    group.bench_function("decode", |bench| {
        bench.iter(|| black_box(CoverageArtifact::from_json(black_box(&json))));
    });

    group.finish();
}

/// Benchmarks deterministic merge scaling and report generation.
fn merge_and_report_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("coverage/merge-report");
    configure_group(&mut group);

    for artifact_count in [1_usize, 8, 64] {
        let (merge, report) = merge_fixture(artifact_count).unwrap_or_else(|_| unreachable!());
        let merge_json = merge.to_json().unwrap_or_default();

        throughput_artifacts(&mut group, u64::try_from(artifact_count).unwrap_or_default());

        group.bench_with_input(BenchmarkId::new("merge", artifact_count), &artifact_count, |bench, _| {
            let artifacts = (0..artifact_count)
                .map(|index| {
                    let status = if index % 3 == 0 {
                        TestStatus::Passed
                    } else if index % 3 == 1 {
                        TestStatus::Failed
                    } else {
                        TestStatus::Error
                    };
                    artifact_with_samples(
                        &format!("bench_merge_{index}"),
                        status,
                        &[(0_u8, 0_u8), (1, 1), (4, 2), (9, 1)],
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_default();

            bench.iter(|| black_box(CoverageMerge::from_artifacts(CoverageMergePolicy::all(), artifacts.clone())));
        });

        group.bench_with_input(BenchmarkId::new("merge-decode", artifact_count), &artifact_count, |bench, _| {
            bench.iter(|| black_box(CoverageMerge::from_json(black_box(&merge_json))));
        });

        group.bench_with_input(BenchmarkId::new("report-text", artifact_count), &artifact_count, |bench, _| {
            bench.iter(|| black_box(report.to_text()));
        });

        group.bench_with_input(BenchmarkId::new("report-html", artifact_count), &artifact_count, |bench, _| {
            bench.iter(|| black_box(report.to_html()));
        });
    }

    group.finish();
}

criterion_group!(coverage_benches, coverage_sampling_benches, artifact_benches, merge_and_report_benches);
criterion_main!(coverage_benches);
