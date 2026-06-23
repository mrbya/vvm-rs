#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk",
)]
struct MockClock(bool);

struct MockDut;

fn main() {}
