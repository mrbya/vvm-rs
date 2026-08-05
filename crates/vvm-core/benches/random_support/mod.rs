//! Random benchmark support.

use std::time::Duration;

use criterion::{BenchmarkGroup, Throughput, measurement::WallTime};
use vvm_core::{RandomContext, Randomize, ReplayToken, ReplayableSequence, Seed};

/// Fixed benchmark seed shared across deterministic random workloads.
pub const BENCH_SEED: Seed = Seed::new(0x5a17_d3c4_92ef_1001);

/// Stable replay token shared by replayable benchmark sequences.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(BENCH_SEED);

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

/// Deterministic randomizable transaction used by replay benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomTransaction {
    /// One Boolean flag.
    pub enable: bool,
    /// One 32-bit payload.
    pub payload: u32,
    /// One signed scalar payload.
    pub signed: i64,
}

impl Randomize for RandomTransaction {
    fn randomize(random: &mut RandomContext) -> Self {
        Self {
            enable: bool::randomize(random),
            payload: u32::randomize(random),
            signed: i64::randomize(random),
        }
    }
}

/// Deterministic replayable transaction stream.
#[derive(Debug, Clone)]
pub struct RandomTransactionSequence {
    /// Replay metadata used to reconstruct the stream.
    replay: ReplayToken,
    /// Number of items to generate.
    len: usize,
}

impl RandomTransactionSequence {
    /// Creates one deterministic replayable sequence description.
    #[must_use]
    pub const fn new(replay: ReplayToken, len: usize) -> Self {
        Self { replay, len }
    }
}

impl ReplayableSequence for RandomTransactionSequence {
    fn replay_token(&self) -> ReplayToken {
        self.replay
    }
}

impl IntoIterator for RandomTransactionSequence {
    type Item = RandomTransaction;
    type IntoIter = std::vec::IntoIter<RandomTransaction>;

    fn into_iter(self) -> Self::IntoIter {
        let mut random = RandomContext::from_replay(self.replay);
        let mut values = Vec::with_capacity(self.len);

        for _ in 0..self.len {
            values.push(RandomTransaction::randomize(&mut random));
        }

        values.into_iter()
    }
}
