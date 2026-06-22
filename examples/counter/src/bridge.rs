//! Handwritten safe wrapper around the generated counter bridge.

use cxx::UniquePtr;

include!(concat!(env!("OUT_DIR"), "/vvm/counter/generated/bridge.rs"));

/// Local result type for the handwritten counter wrapper.
pub type Result<T> = std::result::Result<T, &'static str>;

/// Safe local wrapper around the handwritten CXX bridge.
pub struct Counter {
    /// Opaque ownership of the handwritten C++ adapter.
    inner: UniquePtr<ffi::Counter>,

    /// Tracks whether `finish()` has already been forwarded to C++.
    finished: bool,
}

impl Counter {
    /// Constructs a Verilated counter model.
    pub fn new() -> Result<Self> {
        let inner = ffi::create_counter();

        if inner.is_null() {
            return Err("failed to construct the Verilated counter model");
        }

        Ok(Self {
            inner,
            finished: false,
        })
    }

    /// Evaluates the current model state.
    pub fn eval(&mut self) {
        self.inner_mut().eval();
    }

    /// Finalises the model exactly once.
    pub fn finish(&mut self) {
        if self.finished {
            return;
        }

        self.inner_mut().finish();
        self.finished = true;
    }

    /// Drives the clock input.
    pub fn set_clk(&mut self, value: bool) {
        self.inner_mut().set_clk(value);
    }

    /// Drives the active-low reset input.
    pub fn set_reset_n(&mut self, value: bool) {
        self.inner_mut().set_reset_n(value);
    }

    /// Drives the enable input.
    pub fn set_enable(&mut self, value: bool) {
        self.inner_mut().set_enable(value);
    }

    /// Samples the counter output.
    pub fn count(&self) -> Result<u8> {
        self.inner
            .as_ref()
            .map(ffi::Counter::count)
            .ok_or("counter model unexpectedly became null")
    }

    /// Returns a pinned mutable reference to the underlying model.
    fn inner_mut(&mut self) -> std::pin::Pin<&mut ffi::Counter> {
        self.inner.pin_mut()
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        self.finish();
    }
}
