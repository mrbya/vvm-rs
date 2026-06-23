#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = "clk",
)]
enum MockClock {
    High,
    Low,
}

struct MockDut;

fn main() {}
