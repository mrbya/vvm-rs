# Generated Type Mapping

This chapter explains how common HDL port shapes appear in VVM's generated Rust
wrapper.

Always treat the generated Rust type as the contract. When your intuition about
the HDL shape and the generated Rust surface disagree, trust the generated type
and the compiler diagnostics rather than guessing.

## Common Mappings

| HDL shape | Typical Rust shape | Notes |
| --- | --- | --- |
| scalar `bit` or `logic` | `bool` | common for enables resets and flags |
| small unsigned packed values | `u8`, `u16`, `u32`, `u64` | exact width depends on fit |
| small signed packed values | `i8`, `i16`, `i32`, `i64` | preserve signed interpretation |
| wider packed values | packed helper types | used when native integers are too small |
| supported packed arrays | generated packed representations | preserve packed layout semantics |
| supported packed structs | generated packed struct types | field ordering follows the generated contract |
| supported packed enums | generated enum-like representations | use the generated type rather than raw integers |
| supported unpacked arrays | generated collection-like accessors or generated wrappers | verify exact generated shape |
| top-level `inout` | `vvm::dut::InoutState<T>` | caller owns resolution policy |

## Boolean And Scalar Ports

Single-bit public ports usually appear as `bool`. This is the most common shape
for clocks, enables, resets, valids, readies, and flags.

## Signed Values

Signed packed values map to signed Rust integers when the width fits the native
types. Treat the generated signed getter and setter as authoritative, especially
when an HDL declaration changed recently.

## Wide Packed Values

When a packed value does not fit comfortably into Rust's native integer types,
VVM uses packed helper types instead of silently truncating the value.

## Packed Arrays Structs And Enums

Supported packed arrays, structs, and enums are emitted as generated Rust
representations that preserve the external data contract.

Practical rules:

- use the generated field names and ordering;
- derive `Drive` and `Sample` against the generated shape you actually received;
- do not assume a hand-written mirror type will always match the generator.

## Unpacked Arrays

Supported unpacked arrays are exposed through generated representations or
accessors appropriate for the generated wrapper. Check the wrapper surface and
derive diagnostics rather than assuming the packed rules also apply here.

## Inout Ports

Top-level `inout` ports are exposed as `InoutState<T>`. This is intentionally a
caller-owned model.

- driving is explicit;
- sampling is explicit;
- floating and contention policy are your responsibility;
- two-state behavior still applies at the Rust boundary.

## Setter And Getter Behaviour

The generated getters and setters are the authoritative boundary for width,
layout, and supported operations.

- `Drive` uses the generated setters for mapped fields.
- `Sample` uses the generated getters for mapped fields.
- explicit port mapping attributes let you match your Rust field names to the
  generated wrapper names.

## Unsupported Shapes And Diagnostics

When a shape is unsupported or still only covered by fixture-level validation,
the correct response is to treat that as a real limitation.

Typical symptoms:

- `Drive` or `Sample` derive diagnostics about unsupported field types;
- missing generated accessors for a shape you expected;
- compile-time failures when a custom mirror type does not match the generated
  contract.

## Validation References

The public generator-backed validation fixtures currently live under:

- `tests/fixtures/native-signed-adder/`
- `tests/fixtures/native-packed-array/`
- `tests/fixtures/native-packed-struct/`
- `tests/fixtures/native-packed-enum/`
- `tests/fixtures/native-unpacked-array/`
- `tests/fixtures/native-wide-transform/`

These are validation references, not tutorials.

## Related Material

- [Including The DUT](including-the-dut.md)
- [Driving Inputs](driving-inputs.md)
- [Sampling Outputs](sampling-outputs.md)
- [Bidirectional Ports](bidirectional-ports.md)
- [Drive And Sample API Guide](../api-guide/drive-and-sample.md)
- [Inout API Guide](../api-guide/inout.md)
