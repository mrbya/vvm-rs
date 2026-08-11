# FFI Safety Audit

This document records the audited Rust/C++/Verilator boundary for the `v0.2.0`
release line.

Milestone `12.8` treats this as the developer-facing inventory of the generated
DUT wrapper boundary, not as a new public ABI commitment.

## Scope

Audited sources:

- `crates/vvm-build/src/codegen/cxx_bridge.rs`
- `crates/vvm-build/src/codegen/cpp_adapter.rs`
- `crates/vvm-build/src/codegen/rust_wrapper.rs`
- `crates/vvm-core/src/timing.rs`
- `crates/vvm-build/tests/fixtures/codegen/`
- `tests/fixtures/native-*`
- `examples/tri-state-bus/`
- `examples/timed-uart/`

## Inventory Summary

Handwritten Rust `unsafe` blocks in the audited DUT-wrapper path: none.

Generated Rust `unsafe` blocks: none.

Generated Rust `unsafe` boundary declarations: `unsafe extern "C++"` blocks in
the generated `bridge.rs` output emitted by `cxx_bridge.rs`.

Handwritten C++ exception containment: factory construction and trace opening
use explicit `try` / `catch (...)` in generated adapter code.

Native adapter methods exposed across CXX are `noexcept` and rely on the
underlying Verilator operations being non-throwing.

## Boundary Inventory

| Boundary | Owner | Safety argument | Enforcement | Regression coverage |
| --- | --- | --- | --- | --- |
| Generated CXX bridge module (`#[cxx::bridge]`) | `cxx_bridge.rs` | Rust only sees an opaque native adapter type and typed method signatures; no raw pointers cross into safe user code | `unsafe extern "C++"`, opaque C++ type, generated private `ffi` module | `crates/vvm-build/src/codegen/mod.rs`, checked-in bridge snapshots |
| Native DUT ownership | Generated Rust wrapper + generated C++ adapter | One Rust wrapper owns one C++ adapter, and one adapter owns one `VerilatedContext` plus one model instance | Rust stores one `UniquePtr<ffi::Dut>`; C++ adapter deletes copy and move operations and stores `std::unique_ptr<Impl>` | codegen snapshots, native fixtures, examples |
| Pinned mutable native access | Generated Rust wrapper | Mutable native methods require a stable native address while CXX holds `self: Pin<&mut T>` | `inner_mut()` returns `Pin<&mut ffi::Dut>` from `UniquePtr::as_mut()`; mutating bridge methods all require `Pin<&mut T>` | codegen assertions in `codegen/mod.rs`, generated snapshots |
| Immutable native access | Generated Rust wrapper | Read-only queries never create a mutable native alias | `inner_ref()` returns `&ffi::Dut`; generated getters and timing queries use shared references | generated snapshots, native timing fixture |
| Context outlives model | Generated C++ adapter `Impl` | Verilator model receives `context.get()` and must never outlive its context | `Impl` declares `context` before `model`, so destruction drops `model` before `context` | generated C++ snapshots, native timing fixture |
| Finalization exactly once semantically | Generated Rust wrapper + generated C++ adapter | `final()` must not run more than once, and dropping after explicit finish must remain safe | Rust `finished` flag, C++ `impl_->finished` flag, Rust `Drop` and C++ destructor both early-return when already finished | generated snapshots, timing fixture, example tests |
| Trace object lifetime | Generated C++ adapter + generated Rust wrapper | Trace must not outlive the model; finish must flush and close it before native destruction | `Impl` owns `std::unique_ptr<VerilatedVcdC>`; `finish()` dumps once more, closes the trace, then marks finished | `counter-vcd` snapshots, timing trace fixture, counter and tri-state trace tests |
| Timed event queries | Generated C++ adapter + `TimingScheduler` | `events_pending()` and `next_time_slot()` must not mutate time, evaluate the DUT, or consume events | methods are `const noexcept`; wrapper exposes read-only queries; scheduler rejects missing, equal-time, and past-time slots | `tests/fixtures/native-timing-delay`, `crates/vvm-core/src/timing.rs` |
| Time advancement | Generated Rust wrapper + generated C++ adapter | Rust shadow time and native `VerilatedContext` time must stay in lockstep | Rust checks `checked_add` before calling native `advance_time`; C++ checks `timeInc` overflow before mutating context time | wrapper generation tests, native timing fixture |
| Scalar port transfer | Generated C++ adapter | Narrow HDL widths must not leak high bits or mis-handle sign | generated masking for narrow unsigned inputs and outputs; generated sign-extension helper for signed outputs | codegen unit tests, signed native fixture |
| Wide packed port transfer | Generated Rust wrapper + generated C++ adapter | Transfers require exact word counts and final-word masking for non-word-aligned widths | generated `words.size() == expected_words` checks, `RawType::Words` static assertions, final-word masks, Rust conversion failure paths | codegen unit tests, wide native fixture |
| Unpacked array transfer | Generated Rust wrapper + generated C++ adapter | Rust and native sides must agree on flattened transfer length and HDL element order | exact transfer-size checks, native array-size check, generated ordinal mapping, Rust chunking and reconstruction with checked failures | unpacked-array fixture, codegen unit tests |
| Packed aggregate transfer | Generated Rust wrapper | Packed arrays, structs, and enums must map through canonical flattened storage without unchecked indexing | conversions route through `words_le()` / `from_words_le(...)` and checked extractors; conversion failures become typed errors | packed-array, packed-struct, packed-enum fixtures |
| Inout split-state transfer | Generated Rust wrapper + generated C++ adapter | Caller-owned resolution requires separate input, enable, and output-value components without hidden native resolution | generated methods for `<port>_input`, `<port>_output_enable`, `<port>_output_value`, plus composite `InoutState` | tri-state example, inout fixtures |
| Exception containment on constructor | Generated C++ adapter | Construction failures must not unwind across CXX into Rust | generated factory function wraps `std::make_unique<...>()` in `try` / `catch (...)` and returns `nullptr` | codegen snapshots, consumer construction tests |
| Exception containment on trace open | Generated C++ adapter | Trace setup may allocate and open files; failures must not unwind across CXX | generated `open_trace()` wraps trace allocation, model hookup, and file open in `try` / `catch (...)` and returns `false` | `counter-vcd` snapshots, trace fixtures |
| Panic containment | Generated Rust wrapper | No Rust callback is exposed to C++, so there is no supported Rust-unwind-to-C++ path in this boundary | generated bridge only calls C++ from Rust; no Rust functions are exported back to C++ | inventory review |
| Thread confinement | Generated Rust wrapper | Generated DUT wrappers are intentionally thread-confined and must not be moved or shared across threads | generated wrapper stores `PhantomData<Rc<()>>`, making the wrapper neither `Send` nor `Sync` | native compile-fail fixture `native-thread-confinement`, codegen tests |

## CXX Operation Inventory

The generated `unsafe extern "C++"` surface contains these operation families:

- construction: `create_<dut>() -> UniquePtr<Dut>`
- lifecycle: `eval`, `advance_time`, `finish`
- timing: `events_pending`, `next_time_slot`
- tracing: `open_trace`, `close_trace`, `trace_is_open`
- scalar input setters
- scalar output getters
- wide packed input setters with `&[u32]`
- wide packed output getters with `&mut [u32]`
- unpacked-array input setters with typed slices
- unpacked-array output getters with mutable typed slices
- inout split-component setters and getters

No `extern "C"` boundary is used in the audited DUT-wrapper path.

## Generated Snapshot Coverage

Representative checked-in generated specimens:

- `crates/vvm-build/tests/fixtures/codegen/counter/`
- `crates/vvm-build/tests/fixtures/codegen/counter-vcd/`
- `crates/vvm-build/tests/fixtures/codegen/packed-enum-ports/`

Representative runtime fixture coverage:

- `tests/fixtures/native-wide-transform/`
- `tests/fixtures/native-packed-array/`
- `tests/fixtures/native-packed-struct/`
- `tests/fixtures/native-packed-enum/`
- `tests/fixtures/native-unpacked-array/`
- `tests/fixtures/native-timing-delay/`
- `examples/tri-state-bus/`

## Audited Assumptions

These assumptions are real parts of the current implementation and must remain
documented:

- Trace paths are UTF-8 only because the CXX bridge uses `&str` / `rust::Str`.
- Trace opening is one-shot per wrapper instance: the Rust wrapper marks tracing
  configured even when native `open_trace()` returns `false`.
- Repeated evaluations at the same simulation time intentionally collapse to one
  VCD dump because `dump_trace()` ignores `now <= last_trace_time` after the
  first dump.
- Rust wrapper time is shadow state. Correctness depends on native time changes
  happening only through generated `advance_time()`.
- `TimingScheduler` intentionally rejects equal-time and past-time delayed slots.
- Inout correctness depends on caller-owned resolution plus bounded repeated
  evaluation at one logical simulation time.

## Findings Needing Follow-up Hardening

The inventory found no handwritten Rust `unsafe` block in the generated DUT
wrapper path.

The inventory did find these assumptions that require explicit follow-up review
or continued documentation:

- adapter methods other than construction and trace opening rely on Verilator and
  surrounding native operations being non-throwing under `noexcept`;
- drop-time finalization is intentionally best-effort and unobservable.

These are classified and documented. Thread confinement is now enforced in the
generated wrapper.
