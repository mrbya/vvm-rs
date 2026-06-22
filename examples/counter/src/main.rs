//! Manual simulation using the generated Verilated counter wrapper.

/// Generated DUT wrapper.
mod generated {
    include!(concat!(env!("OUT_DIR"), "/vvm/counter/generated/dut.rs"));
}

use generated::{Counter, Result};

fn main() -> Result<()> {
    let mut counter = Counter::new()?;
    let mut cycle: u8 = 0;

    counter.set_clk(false);
    counter.set_reset_n(false);
    counter.set_enable(false);
    counter.eval();
    print_count(cycle, "reset low", counter.count()?);
    expect_count(&counter, 0, "count must stay zero while reset is asserted")?;

    counter.set_clk(true);
    counter.eval();
    print_count(cycle, "reset rising edge", counter.count()?);
    expect_count(&counter, 0, "count must stay zero on the reset edge")?;

    counter.set_clk(false);
    counter.set_reset_n(true);
    counter.set_enable(true);
    counter.eval();

    for expected in 1..=3 {
        cycle = cycle.saturating_add(1);

        counter.set_clk(true);
        counter.eval();
        print_count(cycle, "enabled rising edge", counter.count()?);
        expect_count(
            &counter,
            expected,
            "count must increment on enabled rising edges",
        )?;

        counter.set_clk(false);
        counter.eval();
        print_count(cycle, "enabled falling edge", counter.count()?);
        expect_count(
            &counter,
            expected,
            "count must remain stable between rising edges",
        )?;
    }

    counter.set_enable(false);
    counter.eval();

    cycle = cycle.saturating_add(1);
    counter.set_clk(true);
    counter.eval();
    print_count(cycle, "disabled rising edge", counter.count()?);
    expect_count(&counter, 3, "count must not increment while disabled")?;

    counter.set_clk(false);
    counter.eval();
    print_count(cycle, "disabled falling edge", counter.count()?);
    expect_count(
        &counter,
        3,
        "count must remain stable after a disabled cycle",
    )?;

    counter.finish();

    println!("manual counter simulation completed successfully");

    Ok(())
}

/// Prints the observed count for a given simulation step.
fn print_count(cycle: u8, phase: &str, count: u8) {
    println!("cycle {cycle:02} {phase}: count={count}");
}

/// Checks the observed count against the expected value.
fn expect_count(counter: &Counter, expected: u8, message: &'static str) -> Result<()> {
    let actual = counter.count()?;

    if actual == expected {
        return Ok(());
    }

    Err(message)
}
