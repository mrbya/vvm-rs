//! Raw bidirectional-port state snapshots.

/// Raw state of one bidirectional DUT port.
///
/// `InoutState` is intentionally unresolved. VVM exposes the externally applied
/// input value, the DUT output-enable mask, and the DUT-proposed output value so
/// callers can implement their own resolution policy in ordinary Rust.
///
/// This is a two-state Rust-side model. If the HDL design relies on four-state
/// electrical behavior, floating semantics, or analog contention rules, model
/// those effects explicitly at the caller layer rather than expecting VVM to
/// infer them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InoutState<Value, Enable = Value> {
    /// Externally resolved input currently presented to the DUT.
    input: Value,

    /// Per-bit mask indicating which bits the DUT drives.
    output_enable: Enable,

    /// Value proposed by the DUT.
    output_value: Value,
}

impl<Value, Enable> InoutState<Value, Enable> {
    /// Creates an inout-state snapshot.
    pub const fn new(input: Value, output_enable: Enable, output_value: Value) -> Self {
        Self {
            input,
            output_enable,
            output_value,
        }
    }

    /// Returns the externally resolved input.
    #[must_use]
    pub const fn input(&self) -> &Value {
        &self.input
    }

    /// Returns the DUT output-enable mask.
    #[must_use]
    pub const fn output_enable(&self) -> &Enable {
        &self.output_enable
    }

    /// Returns the DUT-proposed output value.
    #[must_use]
    pub const fn output_value(&self) -> &Value {
        &self.output_value
    }

    /// Consumes the snapshot and returns its components.
    #[must_use]
    pub fn into_parts(self) -> (Value, Enable, Value) {
        (self.input, self.output_enable, self.output_value)
    }
}

#[cfg(test)]
mod tests {
    use super::InoutState;
    use crate::Bits;

    #[test]
    fn constructs_and_returns_components() {
        let state = InoutState::<i16, u16>::new(-42, 0x00FF, 21);

        assert_eq!(state.input(), &-42);
        assert_eq!(state.output_enable(), &0x00FF);
        assert_eq!(state.output_value(), &21);
        assert_eq!(state.into_parts(), (-42, 0x00FF, 21));
    }

    #[test]
    fn default_uses_component_defaults() {
        assert_eq!(InoutState::<u8>::default().into_parts(), (0, 0, 0));
    }

    #[test]
    fn primitive_state_is_copy() {
        let state = InoutState::new(true, true, false);
        let copied = state;

        assert_eq!(state, copied);
    }

    #[test]
    fn wide_state_is_cloneable() {
        let state = InoutState::new(Bits::<65>::zero(), Bits::<65>::zero(), Bits::<65>::zero());

        assert_eq!(state, state.clone());
    }
}
