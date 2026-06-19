//! Minimal executable exercising the handwritten Verilated counter adapter.

mod bridge;

fn main() -> Result<(), &'static str> {
    let count = bridge::adapter_smoke_test()?;

    if count != 0 {
        return Err("counter output was not zero while reset was asserted");
    }

    println!("Verilated counter adapter constructed successfully");
    println!("counter output after initial reset evaluation: {count}");

    Ok(())
}
