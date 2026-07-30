# Macro Architecture

`vvm-macros` owns derive and attribute expansion.

Its responsibilities are:

- parse `#[vvm(...)]` inputs;
- expand `Drive`, `Sample`, `Clock`, and `Coverage` derives;
- expand `#[vvm::test]` into normal Rust test registration;
- emit compile-time diagnostics through `syn::Error`-style reporting.

The trybuild suite is the public contract for many macro diagnostics and should
be updated deliberately.
