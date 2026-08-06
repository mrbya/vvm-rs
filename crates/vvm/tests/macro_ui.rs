//! Compile-pass and compile-fail tests for the public VVM macro surface.

use std::path::PathBuf;

#[test]
fn derive_ui() {
    let macro_fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("vvm crate should be nested under crates/")
        .parent()
        .expect("workspace root should contain crates/")
        .join("crates/vvm-macros/tests/ui");

    let pass_cases = macro_fixture_root.join("pass/*.rs");
    let fail_cases = macro_fixture_root.join("fail/*.rs");
    let cases = trybuild::TestCases::new();

    cases.pass(pass_cases.to_string_lossy().as_ref());
    cases.compile_fail(fail_cases.to_string_lossy().as_ref());
}
