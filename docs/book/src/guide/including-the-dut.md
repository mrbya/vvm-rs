# Including The DUT

`vvm::include_dut!` includes the generated Rust wrapper from `OUT_DIR` into your
test code.

The practical rules are simple:

- use the same logical DUT name that you passed to `DutBuilder::new`;
- keep the inclusion inside test code or a test-only module when the wrapper is
  test-only;
- import the generated type from the included module, not from internal build
  artifacts.

If the names do not match, the include path will be wrong and the compile will
fail before your tests ever run.
