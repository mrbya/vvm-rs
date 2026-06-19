//! Handwritten CXX bridge declarations for the counter example scaffold.

#[cxx::bridge]
/// Raw CXX bridge bindings for the handwritten counter scaffold.
mod ffi {
    unsafe extern "C++" {
        include!("counter.hpp");

        /// Returns the width of the counter output port.
        fn count_width() -> u8;

        /// Returns whether the reset input is active low.
        fn reset_is_active_low() -> bool;
    }
}

/// Returns the width of the counter output port.
#[must_use]
pub fn count_width() -> u8 {
    ffi::count_width()
}

/// Returns whether the reset input is active low.
#[must_use]
pub fn reset_is_active_low() -> bool {
    ffi::reset_is_active_low()
}
