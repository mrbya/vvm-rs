# Generated Types

This page summarizes the main HDL-to-Rust shapes exposed by VVM's generated
wrappers.

## Common Supported Shapes

| HDL shape | Typical Rust shape | Notes |
| --- | --- | --- |
| scalar bit/logic | `bool` | common for enables, resets, and flags |
| small unsigned packed values | `u8`, `u16`, `u32`, `u64` | exact width mapping depends on fit |
| small signed packed values | `i8`, `i16`, `i32`, `i64` | see the signed-adder fixture for public usage |
| wider packed values | packed helper types | use packed utilities when native integers are too small |
| supported packed arrays, structs, and enums | generated packed representations | validate against current generator support |
| top-level `inout` | `vvm::dut::InoutState<T>` | caller owns resolution policy |

## Practical Rule

Always treat the generated Rust type as the contract, not your intuition about
how the HDL shape "should probably map".

## Unsupported Or Awkward Shapes

Some port shapes still rely on fixture-level validation rather than a polished
general user workflow. When a shape is not documented or the derive mapping does
not compile cleanly, treat that as a real limitation and verify it against the
native fixtures instead of guessing.

## Backing Fixtures

The most precise generator-backed public regressions currently live under:

- `tests/fixtures/native-signed-adder/`
- `tests/fixtures/native-packed-array/`
- `tests/fixtures/native-packed-struct/`
- `tests/fixtures/native-packed-enum/`
- `tests/fixtures/native-unpacked-array/`
- `tests/fixtures/native-wide-transform/`

## Related Material

- [Drive And Sample API Guide](../api-guide/drive-and-sample.md)
- [Bidirectional Ports](../guide/bidirectional-ports.md)
- [Limitations](limitations.md)
