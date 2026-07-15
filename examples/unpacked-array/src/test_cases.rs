use crate::unpacked_array_ports::{Bytes, Flags, SignedValues, UnpackedArrayPorts, WideValues};

#[test]
fn unpacked_array_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let mut dut = UnpackedArrayPorts::new()?;
    let flags = Flags::from_array([false, true, false, true]);
    let bytes = Bytes::from_array([0x33, 0x22, 0x11, 0x00]);
    let signed = SignedValues::from_array([-1, 0, 123]);
    let first = vvm::Bits::<129>::from_words_le([0xfeed_beef, 0, 0, 0, 0])?;
    let second = vvm::Bits::<129>::from_words_le([0, 0, 0, 0, 1])?;
    let wide = WideValues::from_array([first, second]);
    dut.set_flags(&flags)?;
    dut.set_bytes(&bytes)?;
    dut.set_signed_values(&signed)?;
    dut.set_wide_values(&wide)?;
    dut.set_clk(false)?;
    dut.eval()?;
    dut.set_clk(true)?;
    dut.eval()?;
    assert_eq!(dut.flags_out()?.as_array(), flags.as_array());
    assert_eq!(dut.bytes_out()?.as_array(), bytes.as_array());
    assert_eq!(dut.signed_values_out()?.as_array(), signed.as_array());
    assert_eq!(*dut.bytes_out()?.element(3)?, 0x33);
    assert_eq!(*dut.bytes_out()?.element(0)?, 0x00);
    assert!(!dut.first_flag_out()? && dut.last_flag_out()?);
    assert_eq!(dut.bytes_left_out()?, 0x33);
    assert_eq!(dut.bytes_right_out()?, 0x00);
    assert_eq!(dut.wide_left_low_word_out()?, 0xfeed_beef);
    assert!(dut.wide_right_high_bit_out()?);
    dut.finish()?;
    Ok(())
}
