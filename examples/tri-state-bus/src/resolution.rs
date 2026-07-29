//! Caller-owned push-pull bus resolution and bounded settling.

use std::fmt;

/// One participant's proposed push-pull bus drive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BusDriver {
    /// Bits actively driven by this participant.
    enable: u8,

    /// Values proposed on enabled bits.
    value: u8,
}

impl BusDriver {
    /// Creates one bus-driver proposal.
    #[must_use]
    pub const fn new(enable: u8, value: u8) -> Self {
        Self { enable, value }
    }

    /// Creates a fully released driver.
    #[must_use]
    pub const fn released() -> Self {
        Self::new(0, 0)
    }

    /// Returns the active-drive mask.
    #[must_use]
    pub const fn enable(self) -> u8 {
        self.enable
    }

    /// Returns the proposed value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.value
    }
}

/// Successfully resolved two-state bus value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BusResolution {
    /// Final value presented to the DUT.
    value: u8,

    /// Bits driven by at least one participant.
    driven_mask: u8,

    /// Bits supplied by the floating-value policy.
    floating_mask: u8,
}

impl BusResolution {
    /// Returns the resolved two-state bus value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.value
    }

    /// Returns bits driven by at least one participant.
    #[must_use]
    pub const fn driven_mask(self) -> u8 {
        self.driven_mask
    }

    /// Returns bits supplied by the floating policy.
    #[must_use]
    pub const fn floating_mask(self) -> u8 {
        self.floating_mask
    }
}

/// Per-bit push-pull bus contention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BusContention {
    /// Bits driven to different values by both participants.
    mask: u8,
}

impl BusContention {
    /// Creates one contention diagnostic.
    const fn new(mask: u8) -> Self {
        Self { mask }
    }

    /// Returns the exact contended-bit mask.
    #[must_use]
    pub const fn mask(self) -> u8 {
        self.mask
    }
}

impl fmt::Display for BusContention {
    /// Formats the exact contended-bit mask for user-facing diagnostics.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "tri-state bus contention on mask 0x{:02X}",
            self.mask
        )
    }
}

impl std::error::Error for BusContention {}

/// Resolves DUT and external push-pull drivers.
///
/// `floating_value` supplies values for bits released by both participants.
///
/// # Errors
///
/// Returns [`BusContention`] when both participants drive one or more bits while
/// proposing different values.
pub const fn resolve_bus(
    dut: BusDriver,
    external: BusDriver,
    floating_value: u8,
) -> Result<BusResolution, BusContention> {
    let overlap = dut.enable() & external.enable();
    let differing_values = dut.value() ^ external.value();
    // Only conflicting bits that both sides actively drive are contention.
    let contention = overlap & differing_values;

    if contention != 0 {
        return Err(BusContention::new(contention));
    }

    let driven_mask = dut.enable() | external.enable();
    let floating_mask = !driven_mask;
    let resolved_value = (dut.value() & dut.enable())
        | (external.value() & external.enable())
        | (floating_value & floating_mask);

    Ok(BusResolution {
        value: resolved_value,
        driven_mask,
        floating_mask,
    })
}

/// Result of one successful bus-settling run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettleRun {
    /// Stable bus resolution.
    resolution: BusResolution,

    /// Number of DUT evaluations performed.
    evaluations: usize,
}

impl SettleRun {
    /// Returns the stable bus resolution.
    #[must_use]
    pub const fn resolution(self) -> BusResolution {
        self.resolution
    }

    /// Returns the number of evaluations performed.
    #[must_use]
    pub const fn evaluations(self) -> usize {
        self.evaluations
    }
}

/// Failure while settling the tri-state bus.
#[derive(Debug)]
pub enum SettleError {
    /// Generated DUT operation failed.
    Dut(crate::tri_state_bus::TriStateBusError),

    /// The DUT and external participant drove incompatible values.
    Contention(BusContention),

    /// A zero evaluation limit was supplied.
    InvalidEvaluationLimit,

    /// The interface did not converge within the configured bound.
    EvaluationLimitExceeded {
        /// Maximum number of evaluations attempted.
        limit: usize,
    },
}

impl From<crate::tri_state_bus::TriStateBusError> for SettleError {
    /// Wraps a generated-DUT failure in the settling error domain.
    fn from(error: crate::tri_state_bus::TriStateBusError) -> Self {
        Self::Dut(error)
    }
}

impl From<BusContention> for SettleError {
    /// Wraps deterministic resolution contention in the settling error domain.
    fn from(error: BusContention) -> Self {
        Self::Contention(error)
    }
}

impl fmt::Display for SettleError {
    /// Formats stable diagnostics for each settling failure category.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Dut(error) => write!(formatter, "tri-state DUT operation failed: {error}"),
            Self::Contention(error) => error.fmt(formatter),
            Self::InvalidEvaluationLimit => {
                write!(
                    formatter,
                    "tri-state settling requires a non-zero evaluation limit"
                )
            }
            Self::EvaluationLimitExceeded { limit } => {
                write!(
                    formatter,
                    "tri-state bus did not settle within {limit} evaluations"
                )
            }
        }
    }
}

impl std::error::Error for SettleError {
    /// Returns the generated-DUT or contention source when one exists.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Dut(ref error) => Some(error),
            Self::Contention(ref error) => Some(error),
            Self::InvalidEvaluationLimit | Self::EvaluationLimitExceeded { .. } => None,
        }
    }
}

/// Evaluates and resolves the bus until the presented input is stable.
///
/// This function does not advance simulation time or finalize the DUT.
///
/// # Errors
///
/// Returns an error if DUT evaluation fails, contention is detected, the
/// evaluation bound is zero, or the bus does not converge within the bound.
pub fn settle_bus(
    dut: &mut crate::tri_state_bus::TriStateBus,
    external: BusDriver,
    floating_value: u8,
    max_evaluations: usize,
) -> Result<SettleRun, SettleError> {
    if max_evaluations == 0 {
        return Err(SettleError::InvalidEvaluationLimit);
    }

    for evaluation in 1..=max_evaluations {
        // Evaluate first: the DUT proposal can depend combinationally on the
        // resolved value presented during the preceding iteration.
        dut.eval()?;

        let state = dut.data()?;
        let dut_driver = BusDriver::new(*state.output_enable(), *state.output_value());
        let resolution = resolve_bus(dut_driver, external, floating_value)?;

        if *state.input() == resolution.value() {
            return Ok(SettleRun {
                resolution,
                evaluations: evaluation,
            });
        }

        // A changed resolved input needs another evaluation before declaring
        // convergence; the limit keeps feedback loops bounded.
        dut.set_data_input(resolution.value())?;
    }

    Err(SettleError::EvaluationLimitExceeded {
        limit: max_evaluations,
    })
}

#[cfg(test)]
/// Unit checks for the deterministic push-pull policy.
mod tests {
    use super::{BusDriver, resolve_bus};

    #[test]
    fn resolves_floating_bits() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::released(), BusDriver::released(), 0xFF)?;

        assert_eq!(resolution.value(), 0xFF);
        assert_eq!(resolution.driven_mask(), 0x00);
        assert_eq!(resolution.floating_mask(), 0xFF);

        Ok(())
    }

    #[test]
    fn resolves_external_driver() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::new(0, 0), BusDriver::new(0xFF, 0xA5), 0)?;

        assert_eq!(resolution.value(), 0xA5);
        assert_eq!(resolution.driven_mask(), 0xFF);
        assert_eq!(resolution.floating_mask(), 0);

        Ok(())
    }

    #[test]
    fn resolves_dut_driver() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::new(0xF0, 0xA0), BusDriver::released(), 0xFF)?;

        assert_eq!(resolution.value(), 0xAF);
        assert_eq!(resolution.driven_mask(), 0xF0);
        assert_eq!(resolution.floating_mask(), 0x0F);

        Ok(())
    }

    #[test]
    fn resolves_non_overlapping_drivers() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::new(0xF0, 0xA0), BusDriver::new(0x0F, 0x05), 0)?;

        assert_eq!(resolution.value(), 0xA5);
        assert_eq!(resolution.driven_mask(), 0xFF);
        assert_eq!(resolution.floating_mask(), 0);

        Ok(())
    }

    #[test]
    fn accepts_matching_overlapping_drivers() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::new(0x0F, 0x05), BusDriver::new(0x03, 0x01), 0)?;

        assert_eq!(resolution.value(), 0x05);

        Ok(())
    }

    #[test]
    fn reports_exact_contention_mask() {
        let result = resolve_bus(BusDriver::new(0x0F, 0x0A), BusDriver::new(0x0F, 0x05), 0);

        assert_eq!(result.map_err(super::BusContention::mask), Err(0x0F));
    }

    #[test]
    fn reports_partial_contention_mask() {
        let result = resolve_bus(BusDriver::new(0x0F, 0x09), BusDriver::new(0x0F, 0x05), 0);

        assert_eq!(result.map_err(super::BusContention::mask), Err(0x0C));
    }

    #[test]
    fn ignores_dut_value_on_disabled_bits() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::new(0xF0, 0xAF), BusDriver::released(), 0x05)?;

        assert_eq!(resolution.value(), 0xA5);

        Ok(())
    }

    #[test]
    fn ignores_external_value_on_disabled_bits() -> Result<(), Box<dyn std::error::Error>> {
        let resolution = resolve_bus(BusDriver::released(), BusDriver::new(0x0F, 0xF5), 0xA0)?;

        assert_eq!(resolution.value(), 0xA5);

        Ok(())
    }

    #[test]
    fn formats_contention_diagnostic() {
        let result = resolve_bus(BusDriver::new(1, 1), BusDriver::new(1, 0), 0);
        let error = result.err();

        assert_eq!(
            error.map(|contention| contention.to_string()),
            Some(String::from("tri-state bus contention on mask 0x01"))
        );
    }
}
