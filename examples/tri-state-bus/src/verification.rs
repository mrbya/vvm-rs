//! Drive and sample types for the tri-state bus example.

/// DUT-owned tri-state drive controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vvm::Drive)]
#[vvm(dut = crate::tri_state_bus::TriStateBus)]
pub struct DutDriverControl {
    /// Per-bit DUT output-enable mask.
    #[vvm(port)]
    drive_enable: u8,

    /// DUT-proposed bus value.
    #[vvm(port)]
    drive_value: u8,
}

impl DutDriverControl {
    /// Creates DUT drive controls.
    #[must_use]
    pub const fn new(drive_enable: u8, drive_value: u8) -> Self {
        Self {
            drive_enable,
            drive_value,
        }
    }

    /// Creates a released DUT driver.
    #[must_use]
    pub const fn released() -> Self {
        Self::new(0, 0)
    }
}

/// Raw bus state observed after resolution and DUT evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, vvm::Sample)]
#[vvm(dut = crate::tri_state_bus::TriStateBus)]
pub struct BusObservation {
    /// Complete generated inout state.
    // VVM exposes split input, enable, and value components; it does not impose
    // an electrical resolution policy.
    #[vvm(port)]
    data: vvm::dut::InoutState<u8>,

    /// Value sampled inside the HDL DUT.
    #[vvm(port)]
    sampled_data: u8,
}

impl BusObservation {
    /// Returns the complete raw inout snapshot.
    #[must_use]
    pub const fn data(&self) -> &vvm::dut::InoutState<u8> {
        &self.data
    }

    /// Returns the value sampled inside the HDL design.
    #[must_use]
    pub const fn sampled_data(self) -> u8 {
        self.sampled_data
    }
}
