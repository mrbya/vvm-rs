use core::fmt;
use std::str::FromStr;

use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

/// User-provided seed for a deterministic random stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Seed(u64);

impl Seed {
    /// Constructs a seed frpm its raw value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw seed value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for Seed {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

/// Deterministic random-stream algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomAlgorithm {
    /// ChaCha8-based stream version 1.
    ChaCha8V1,
}

impl fmt::Display for RandomAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ChaCha8V1 => f.write_str("chacha8-v1"),
        }
    }
}

/// Complete identifier for a deterministic random stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplayToken {
    /// RNG algorithm and sampling contract.
    algorithm: RandomAlgorithm,

    /// Initial stream seed.
    seed: Seed,
}

impl ReplayToken {
    /// Creates a token using the current VVM random algorithm.
    #[must_use]
    pub const fn new(seed: Seed) -> Self {
        Self {
            algorithm: RandomAlgorithm::ChaCha8V1,
            seed,
        }
    }

    /// Creates a token from explicit components.
    #[must_use]
    pub const fn from_parts(algorithm: RandomAlgorithm, seed: Seed) -> Self {
        Self { algorithm, seed }
    }

    /// Returns the stream seed.
    #[must_use]
    pub const fn seed(self) -> Seed {
        self.seed
    }

    /// Returns the rng algorithm.
    #[must_use]
    pub const fn algorithm(self) -> RandomAlgorithm {
        self.algorithm
    }
}

impl fmt::Display for ReplayToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:016x}", self.algorithm, self.seed.value())
    }
}

/// Error returned when parsing a replay token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseReplayTokenError {
    /// The algorithm/seed separator is missing.
    MissingSeparator,

    /// The algorithm identifier is not supported.
    UnsupportedAlgorithm,

    /// The hexadecimal seed is malformed.
    InvalidSeed,
}

impl FromStr for ReplayToken {
    type Err = ParseReplayTokenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((algorithm, seed)) = value.split_once(':') else {
            return Err(ParseReplayTokenError::MissingSeparator);
        };

        let algorithm = match algorithm {
            "chacha8-v1" => RandomAlgorithm::ChaCha8V1,

            _ => {
                return Err(ParseReplayTokenError::UnsupportedAlgorithm);
            }
        };

        let seed =
            u64::from_str_radix(seed, 16).map_err(|_error| ParseReplayTokenError::InvalidSeed)?;

        Ok(Self::from_parts(algorithm, Seed::new(seed)))
    }
}

/// Deterministic random source owned by one sequence.
///
/// The generator is intentionally not cloneable. Accidentally cloning a
/// random source can create two identical streams that appear independent.
pub struct RandomContext {
    /// Metadata required to reconstruct the stream.
    replay: ReplayToken,

    /// Current named RNG state.
    rng: ChaCha8Rng,
}

impl RandomContext {
    /// Creates a deterministic random source from a seed.
    #[must_use]
    pub fn new(seed: Seed) -> Self {
        Self::from_replay(ReplayToken::new(seed))
    }

    /// Reconstructs a deterministic random source.
    #[must_use]
    pub fn from_replay(replay: ReplayToken) -> Self {
        let rng = match replay.algorithm() {
            RandomAlgorithm::ChaCha8V1 => ChaCha8Rng::seed_from_u64(replay.seed().value()),
        };

        Self { replay, rng }
    }

    /// Returns the token required to reproduce this stream.
    #[must_use]
    pub const fn replay_token(&self) -> ReplayToken {
        self.replay
    }

    /// Generates the next raw 32-bit word.
    #[must_use]
    pub fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    /// Generates the next raw 64-bit word.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Generates one Boolean value.
    ///
    /// This consumes exactly one 32-bit RNG word and uses its least
    /// significant bit.
    #[must_use]
    pub fn next_bool(&mut self) -> bool {
        self.next_u32() & 1 != 0
    }
}

impl fmt::Debug for RandomContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RandomContext")
            .field("replay", &self.replay)
            .finish_non_exhaustive()
    }
}

/// Constructs a value from a deterministic VVM random source.
pub trait Randomize: Sized {
    /// Constructs one randomized value.
    fn randomize(random: &mut RandomContext) -> Self;
}

impl Randomize for bool {
    fn randomize(random: &mut RandomContext) -> Self {
        random.next_bool()
    }
}

impl Randomize for u32 {
    fn randomize(random: &mut RandomContext) -> Self {
        random.next_u32()
    }
}

impl Randomize for u64 {
    fn randomize(random: &mut RandomContext) -> Self {
        random.next_u64()
    }
}

impl Randomize for i32 {
    fn randomize(random: &mut RandomContext) -> Self {
        Self::from_le_bytes(random.next_u32().to_le_bytes())
    }
}

impl Randomize for i64 {
    fn randomize(random: &mut RandomContext) -> Self {
        Self::from_le_bytes(random.next_u64().to_le_bytes())
    }
}

/// Sequence that carries enough metadata to reconstruct its random stream.
pub trait ReplayableSequence: IntoIterator {
    /// Returns the replay token for this sequence.
    fn replay_token(&self) -> ReplayToken;
}
