#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk",
    edge = "sideways"
)]
struct MockClock;

struct MockDut;

fn main() {}
