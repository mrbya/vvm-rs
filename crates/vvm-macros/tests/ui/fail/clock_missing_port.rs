#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
)]
struct MockClock;

struct MockDut;

fn main() {}
