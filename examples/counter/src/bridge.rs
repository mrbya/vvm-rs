//! Handwritten CXX bridge for the Verilated counter adapter.

use cxx::UniquePtr;

/// Local result type for the handwritten counter wrapper.
pub type Result<T> = std::result::Result<T, &'static str>;

#[cxx::bridge(namespace = "vvm::counter")]
/// Raw FFI bindings for the handwritten counter adapter.
mod ffi {
    unsafe extern "C++" {
        include!("counter.hpp");

        /// Opaque Verilated counter adapter.
        type Counter;

        /// Constructs a counter model.
        fn create_counter() -> UniquePtr<Counter>;

        /// Evaluates the counter model.
        fn eval(self: Pin<&mut Counter>);

        /// Finalises the counter model.
        fn finish(self: Pin<&mut Counter>);

        /// Drives the clock input.
        fn set_clk(self: Pin<&mut Counter>, value: bool);

        /// Drives the active-low reset input.
        fn set_reset_n(self: Pin<&mut Counter>, value: bool);

        /// Drives the enable input.
        fn set_enable(self: Pin<&mut Counter>, value: bool);

        /// Samples the counter output.
        fn count(self: &Counter) -> u8;
    }
}

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
