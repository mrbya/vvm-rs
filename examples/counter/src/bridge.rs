//! Handwritten CXX bridge for the Verilated counter adapter.

#[cxx::bridge(namespace = "vvm::counter")]
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

/// Constructs and evaluates the handwritten counter adapter.
///
/// This is deliberately only a bridge-level smoke test. The safe reusable
/// Rust wrapper will be introduced in the next implementation step.
pub(crate) fn adapter_smoke_test() -> Result<u8, &'static str> {
    let mut counter = ffi::create_counter();

    {
        let mut counter_ref = counter
            .as_mut()
            .ok_or("failed to construct the Verilated counter model")?;

        counter_ref.as_mut().set_clk(false);
        counter_ref.as_mut().set_reset_n(false);
        counter_ref.as_mut().set_enable(false);
        counter_ref.as_mut().eval();
    }

    let count = counter
        .as_ref()
        .ok_or("counter model unexpectedly became null")?
        .count();

    {
        let counter_ref = counter
            .as_mut()
            .ok_or("counter model unexpectedly became null")?;

        counter_ref.finish();
    }

    Ok(count)
}
