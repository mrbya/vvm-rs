use std::num::{NonZeroU64, ParseIntError};

use vvm::timing::{SchedulerError, TimingEvent};

use crate::delayed_sequence::DelayedSequenceError;

/// Maximum delayed slots accepted by the finite example.
const MAX_TIME_SLOTS: u64 = 16;

/// Timing-delay example error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Generated DUT operation failed.
    #[error(transparent)]
    Dut(#[from] DelayedSequenceError),

    /// Timing scheduler operation failed.
    #[error(transparent)]
    Timing(#[from] SchedulerError<DelayedSequenceError>),

    /// Trace filesystem operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A VCD timestamp was malformed.
    #[error(transparent)]
    InvalidTimestamp(#[from] ParseIntError),

    /// The fixed slot bound was invalid.
    #[error("timing-delay slot limit must be nonzero")]
    InvalidSlotLimit,

    /// An expected delayed event was absent.
    #[error("expected delayed event ordinal {ordinal}")]
    MissingExpectedEvent {
        /// Expected delayed-event ordinal.
        ordinal: u64,
    },
}

/// Result returned by timing-delay verification operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Returns the finite delayed-slot limit for this example.
pub fn time_slot_limit() -> Result<NonZeroU64> {
    NonZeroU64::new(MAX_TIME_SLOTS).ok_or(Error::InvalidSlotLimit)
}

/// Requires one manually advanced delayed event.
pub fn require_event(event: Option<TimingEvent>, ordinal: u64) -> Result<TimingEvent> {
    event.ok_or(Error::MissingExpectedEvent { ordinal })
}

/// Parses timestamp records from VCD contents.
pub fn parse_vcd_timestamps(contents: &str) -> Result<Vec<u64>> {
    contents
        .lines()
        .filter_map(|line| line.strip_prefix('#'))
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Error::from)
}

/// Returns whether timestamps are strictly increasing.
#[must_use]
pub fn timestamps_increase(timestamps: &[u64]) -> bool {
    timestamps
        .windows(2)
        .all(|pair| match (pair.first(), pair.get(1)) {
            (Some(previous), Some(next)) => previous < next,
            _ => false,
        })
}
