# Rust Essentials For HDL Engineers

You do not need to become a general Rust expert before using VVM, but a few Rust
and Cargo concepts appear immediately in almost every project.

## Cargo Package And Crate

A Cargo package is the project directory described by `Cargo.toml`.

A crate is the Rust compilation unit produced from that package. In practice,
VVM users usually start with one library crate that contains:

- `build.rs`;
- HDL sources;
- verification code and tests.

## `Cargo.toml`

`Cargo.toml` declares package metadata and dependencies.

For VVM projects, the first important sections are usually:

- `[dev-dependencies]` for test-only Rust code such as `vvm-rs`;
- `[build-dependencies]` for build-time code such as `vvm-build`.

## `build.rs`

`build.rs` is Cargo's build-time preparation hook.

For VVM, this is where you ask `vvm-build` to run Verilator and generate the DUT
wrapper before the crate itself compiles.

## Modules

A module is just a named Rust namespace. In VVM projects, you might keep helper
code in modules such as `verification`, `coverage`, or `test_cases` so related
types stay together.

## Structs

A struct is a named data shape.

In VVM, structs often represent:

- one stimulus transaction;
- one observation;
- one reference-model state object;
- one coverage model.

## Traits

A trait is a behavioural interface.

For example, a VVM reference model implements the `ReferenceModel` trait to say:
"given this stimulus, I can predict the expected observation".

## Derive Macros

A derive macro generates implementation glue for a struct.

In VVM, derives such as `vvm::Drive`, `vvm::Sample`, and `vvm::Clock` generate
the repetitive code that connects your Rust type to the generated DUT wrapper.

## Iterators

An iterator is a value-producing stream.

For HDL verification, the easiest way to think about it is: an iterator is a
source of the next stimulus transaction.

## `Result`

`Result<T, E>` means an operation either succeeded with `T` or failed with a
diagnostic value `E`.

VVM uses `Result` heavily because generated DUT operations, build steps, trace
setup, and filesystem work can all fail in ways the test should report clearly.

## Unit And Integration Tests

Rust has ordinary unit and integration tests.

VVM keeps that model. `#[vvm::test]` integrates with Cargo's test discovery; it
does not replace it with a separate proprietary runner.

## Environment Variables

Environment variables are shell-provided settings such as `VVM_TRACE_DIR` or
`VVM_REPLAY` that modify a test run without changing the source code.

## Generic Types

Generic types are parameterized over another type.

You do not need to write advanced generic code to use VVM, but you will see it
in API names such as `TestResult<Stimulus, Failure, Error>`, which means the same
result structure can describe many DUT-specific testbenches.

For deeper Rust learning, use the official [Rust Book](https://doc.rust-lang.org/book/title-page.html). For VVM-specific workflow, continue to [Installation](installation.md) or [Quick Start](quick-start.md).
