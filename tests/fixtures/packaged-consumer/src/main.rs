use vvm::coverage::{Bin, Coverpoint};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut coverpoint = Coverpoint::builder("value")
        .bin(Bin::value("zero", 0_u8))
        .build()?;

    let sample = coverpoint.sample(&0)?;
    assert!(sample.hit());

    Ok(())
}
