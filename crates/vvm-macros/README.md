# vvm-macros

`vvm-macros` contains the procedural macro implementation crate for VVM.

Normal user crates should import derives and the test attribute from `vvm`, not
from `vvm-macros` directly.

## What Lives Here

- `Drive`, `Sample`, `Clock`, and `Coverage` derive implementations
- the `#[vvm::test]` attribute expansion support
- compile-time attribute parsing and diagnostics for VVM macro inputs

## Direct Usage Guidance

This crate is published because downstream package resolution requires it, but it
is not the recommended entry point for ordinary verification code.

## Documentation

- Facade overview: <https://byacrates.gitlab.io/vvm-rs/>
- Macro-facing API reference: <https://byacrates.gitlab.io/vvm-rs/api/vvm_macros/>
- Public authoring workflow: <https://byacrates.gitlab.io/vvm-rs/guide/project-setup.html>
