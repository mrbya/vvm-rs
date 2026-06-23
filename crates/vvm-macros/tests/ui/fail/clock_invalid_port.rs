#[derive(vvm_macros::Clock)]
#[vvm(
    dut = MockDut,
    clock = ".af##1",
)]
struct MockClock;

struct MockDut;

fn main() {}
