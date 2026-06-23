#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk",
)]
struct MockClock {
    something: bool,
}

struct MockDut;

fn main() {}
