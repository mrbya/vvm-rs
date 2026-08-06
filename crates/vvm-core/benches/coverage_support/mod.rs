//! Coverage benchmark support.

use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Throughput};
use vvm_core::{
    Bin, CoverageArtifact, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
    CoverageItemRef, CoverageMerge, CoverageMergePolicy, CoverageReport, CoverageSession,
    Coverpoint, CoverpointSample, Cross2, ReplayToken, Seed, TestStatus,
};

/// Fixed benchmark seed shared across deterministic coverage workloads.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x5a17_d3c4_92ef_1001));

/// Applies the default Criterion configuration for fast in-process benchmarks.
pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
}

/// Sets artifact throughput on one benchmark group.
pub fn throughput_artifacts(group: &mut BenchmarkGroup<'_, WallTime>, artifacts: u64) {
    group.throughput(Throughput::Elements(artifacts));
}

/// Sets sample throughput on one benchmark group.
pub fn throughput_samples(group: &mut BenchmarkGroup<'_, WallTime>, samples: u64) {
    group.throughput(Throughput::Elements(samples));
}

/// Deterministic coverage fixture with two coverpoints and one cross.
pub struct CoverageFixture {
    /// Stable coverage instance metadata.
    instance: CoverageGroupInstance,
    /// Operation coverpoint.
    operation: Coverpoint<u8>,
    /// Response coverpoint.
    response: Coverpoint<u8>,
    /// Cross coverage.
    cross: Cross2,
}

impl CoverageFixture {
    /// Creates one deterministic coverage fixture.
    pub fn new(instance_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let operation = Coverpoint::builder("operation")
            .bin(Bin::value("zero", 0_u8))
            .bin(Bin::value("one", 1_u8))
            .bin(Bin::inclusive_range("low", 2_u8, 7_u8))
            .bin(Bin::inclusive_range("high", 8_u8, 15_u8))
            .build()?;
        let response = Coverpoint::builder("response")
            .bin(Bin::value("idle", 0_u8))
            .bin(Bin::value("busy", 1_u8))
            .bin(Bin::value("done", 2_u8))
            .build()?;
        let cross = Cross2::builder("operation_x_response", &operation, &response).build()?;

        Ok(Self {
            instance: CoverageGroupInstance::new("bench_fixture", instance_path)?,
            operation,
            response,
            cross,
        })
    }

    /// Samples one deterministic pair.
    pub fn sample_pair(&mut self, left: u8, right: u8) -> Result<(), Box<dyn std::error::Error>> {
        let left_sample: CoverpointSample = self.operation.sample(&left)?;
        let right_sample: CoverpointSample = self.response.sample(&right)?;

        self.cross.sample(&left_sample, &right_sample)?;

        Ok(())
    }
}

impl CoverageGroup for CoverageFixture {
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        visitor.visit(CoverageItemRef::coverpoint(&self.operation));
        visitor.visit(CoverageItemRef::coverpoint(&self.response));
        visitor.visit(CoverageItemRef::cross2(&self.cross));
    }
}

/// Builds one deterministic coverage artifact from one sampled fixture.
pub fn artifact_with_samples(
    test_name: &str,
    status: TestStatus,
    pairs: &[(u8, u8)],
) -> Result<CoverageArtifact, Box<dyn std::error::Error>> {
    let mut fixture = CoverageFixture::new(&format!("dut.{test_name}"))?;

    for &(left, right) in pairs {
        fixture.sample_pair(left, right)?;
    }

    let mut session = CoverageSession::new(test_name.to_owned())?;
    session.capture(&fixture)?;
    let snapshot = session.finish().ok_or("coverage session was empty")?;

    Ok(CoverageArtifact::from_session(
        status,
        Some(BENCH_REPLAY),
        snapshot,
    )?)
}

/// Builds one deterministic coverage merge and its default report.
pub fn merge_fixture(
    artifact_count: usize,
) -> Result<(CoverageMerge, CoverageReport<'static>), Box<dyn std::error::Error>> {
    let pairs = [(0_u8, 0_u8), (1, 1), (4, 2), (9, 1), (3, 2), (8, 0)];
    let artifacts = (0..artifact_count)
        .map(|index| {
            let status = if index % 3 == 0 {
                TestStatus::Passed
            } else if index % 3 == 1 {
                TestStatus::Failed
            } else {
                TestStatus::Error
            };

            artifact_with_samples(&format!("bench_{index}"), status, &pairs)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let merge = CoverageMerge::from_artifacts(CoverageMergePolicy::all(), artifacts)?;
    let leaked = Box::leak(Box::new(merge.clone()));

    Ok((merge, CoverageReport::new(leaked)))
}
