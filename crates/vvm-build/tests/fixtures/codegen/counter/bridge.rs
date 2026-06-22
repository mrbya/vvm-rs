#[cxx::bridge(namespace = "vvm::counter")]
/// Raw generated FFI bindings for the Verilated DUT adapter.
mod ffi {
    unsafe extern "C++" {
        include!("counter.hpp");

        /// Opaque generated Verilated DUT adapter.
        type Counter;

        /// Constructs a Verilated DUT adapter.
        fn create_counter() -> UniquePtr<Counter>;

        /// Evaluates the current DUT state.
        fn eval(self: Pin<&mut Counter>);

        /// Finalises the DUT model.
        fn finish(self: Pin<&mut Counter>);

        /// Drives the `clk` DUT input.
        fn set_clk(self: Pin<&mut Counter>, value: bool);

        /// Drives the `reset_n` DUT input.
        fn set_reset_n(self: Pin<&mut Counter>, value: bool);

        /// Drives the `enable` DUT input.
        fn set_enable(self: Pin<&mut Counter>, value: bool);

        /// Samples the `count` DUT output.
        fn count(self: &Counter) -> u8;
    }
}
