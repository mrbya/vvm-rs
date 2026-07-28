use core::fmt;
use std::str::FromStr;

use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use thiserror::Error;

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

impl From<i64> for Seed {
    fn from(value: i64) -> Self {
        Self::new(u64::from_le_bytes(value.to_le_bytes()))
    }
}

impl From<u128> for Seed {
    fn from(value: u128) -> Self {
        let bytes = value.to_le_bytes();
        let (low, _) = bytes.split_at(8);
        Self::new(u64::from_le_bytes(low.try_into().unwrap_or_default()))
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl FromStr for Seed {
    type Err = ParseSeedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .map_or_else(|| s.parse::<u64>(), |hex| u64::from_str_radix(hex, 16));

        parsed.map(Self::new).map_err(|_error| ParseSeedError {
            string: s.to_owned(),
        })
    }
}

/// Random seed parsing failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("failed to parse random seed from `{string}`")]
pub struct ParseSeedError {
    /// String random seed was parsed from.
    string: String,
}

impl ParseSeedError {
    /// Returns the invalid seed input.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.string
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
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ParseReplayTokenError {
    /// The algorithm/seed separator is missing.
    #[error("`{string}` is missing `:` separator")]
    MissingSeparator {
        /// String replay token was parsed from.
        string: String,
    },

    /// The algorithm identifier is not supported.
    #[error("`{algo}` algorithm is not supported by VVM randomization")]
    UnsupportedAlgorithm {
        /// String algorithm was parsed from.
        algo: String,
    },

    /// The hexadecimal seed is malformed.
    #[error("`{seed}` is not a valid seed value")]
    InvalidSeed {
        /// String seed was parsed from.
        seed: String,
    },
}

impl FromStr for ReplayToken {
    type Err = ParseReplayTokenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((algorithm, seed)) = value.split_once(':') else {
            return Err(ParseReplayTokenError::MissingSeparator {
                string: value.to_owned(),
            });
        };

        let algorithm = match algorithm {
            "chacha8-v1" => RandomAlgorithm::ChaCha8V1,

            _ => {
                return Err(ParseReplayTokenError::UnsupportedAlgorithm {
                    algo: algorithm.to_owned(),
                });
            }
        };

        let seed =
            u64::from_str_radix(seed, 16).map_err(|_error| ParseReplayTokenError::InvalidSeed {
                seed: seed.to_owned(),
            })?;

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

#[cfg(test)]
mod tests {
    use crate::{ParseReplayTokenError, RandomContext, Randomize, ReplayToken, Seed};

    #[test]
    fn chacha8_v1_stream_is_stable() {
        let mut random = RandomContext::new(Seed::new(0x0123_4567_89ab_cdef));

        assert_eq!(
            [
                random.next_u64(),
                random.next_u64(),
                random.next_u64(),
                random.next_u64(),
            ],
            [
                16_439_174_753_831_872_311,
                8_357_938_314_514_734_176,
                16_425_382_708_020_085_233,
                5_893_040_097_532_677_258,
            ],
        );
    }

    #[test]
    fn replay_token_display_and_parse() {
        let token = ReplayToken::new(Seed::new(0x0123_4567_89ab_cdef));

        assert_eq!(token.to_string().parse::<ReplayToken>(), Ok(token));
    }

    #[test]
    fn seed_display_and_parse() {
        let seed = Seed::new(0x0123_4567_89ab_cdef);

        assert_eq!(seed.to_string().parse::<Seed>(), Ok(seed));
    }

    #[test]
    fn parsing_rejects_malformed_seed_and_replay_inputs() {
        let seed_error = "not-a-seed"
            .parse::<Seed>()
            .expect_err("invalid decimal seed must fail");
        assert_eq!(seed_error.input(), "not-a-seed");

        assert!(matches!(
            "chacha8-v1".parse::<ReplayToken>(),
            Err(ParseReplayTokenError::MissingSeparator { .. })
        ));
        assert!(matches!(
            "other:01".parse::<ReplayToken>(),
            Err(ParseReplayTokenError::UnsupportedAlgorithm { .. })
        ));
        assert!(matches!(
            "chacha8-v1:invalid".parse::<ReplayToken>(),
            Err(ParseReplayTokenError::InvalidSeed { .. })
        ));
    }

    #[test]
    fn replay_reconstructs_randomize_contract() {
        let replay = ReplayToken::new(Seed::new(42));
        let mut original = RandomContext::from_replay(replay);
        let mut replayed = RandomContext::from_replay(original.replay_token());

        assert_eq!(
            bool::randomize(&mut original),
            bool::randomize(&mut replayed)
        );
        assert_eq!(u32::randomize(&mut original), u32::randomize(&mut replayed));
        assert_eq!(i64::randomize(&mut original), i64::randomize(&mut replayed));
    }
}
