# Architecture

VVM is a Rust workspace split into smaller crates with clear ownership boundaries. 

## Workspace Layout

```text
vvm-rs
├── ci              # Dockerfiles
│   └── ...
├── crates          # Workspace crates
│   ├── cargo-vvm
│   │   └── ...
│   ├── vvm
│   │   └── ...
│   ├── vvm-build
│   │   └── ...
│   ├── vvm-core
│   │   └── ...
│   └── vvm-macros
│       └── ...
├── docs
│   ├── book        # Book sources and config
│   │   └── ...
│   └── dev         # Misc developer docs
│       └── ...
├── examples        # Curated examples
│   └── ...
├── scripts         # Helper scripts
│   └── ...
├── tests           # Test fixtures and misc test-related sources
│   └── ...
└── ...

```

## Crate Map

| Crate | Responsibility | What belongs here | What should not live here |
| --- | --- | --- | --- |
| `vvm-rs` | public facade and exports | user-facing modules, derives, docs-facing paths | core implementation details |
| `vvm-core` | runtime primitives | testbench, timing, packed values, coverage internals | facade ergonomics and build-script logic |
| `vvm-build` | build-time generation | Verilator execution, metadata parsing, code generation | runtime verification policy |
| `vvm-macros` | proc macros | derive parsing, expansion, diagnostics | runtime data structures |
| `cargo-vvm` | binary-only orchestration | CLI parsing, child command orchestration, report writing | public library API contracts |

Examples and fixtures prove consumer workflows and code-generation contracts; do
not treat them as dumping grounds for production logic that belongs in a crate.

## Build Pipeline

The build pipeline is:

```text
consumer build.rs
  -> DutBuilder validation
  -> Verilator invocation
  -> metadata parsing and normalization
  -> type mapping
  -> C++ adapter generation
  -> CXX bridge generation
  -> Rust wrapper generation
  -> native compilation
  -> OUT_DIR inclusion by the consumer crate
```

`BuildStage` exists so failures can identify which step broke.

## Code Generation

Code generation lives in `vvm-build` because it is part of the build-time
contract, not the runtime testbench contract.

Its responsibilities include:

- normalizing Verilator metadata into stable internal models;
- mapping supported HDL shapes onto Rust-facing representations;
- generating the private C++ adapter and CXX boundary;
- generating the Rust wrapper included by the consumer.

The generated wrapper is intentionally the public result. The adapter internals
are not a supported user-facing ABI.

## Runtime

At runtime, VVM owns a clear lifecycle:

1. construct the generated DUT wrapper;
2. initialize any trace or timing state;
3. drive inputs;
4. evaluate the DUT and scheduler-owned transitions;
5. sample outputs;
6. predict and compare;
7. record coverage;
8. finalize and close native resources.

Keeping these phases separate is why the codebase cares about clear abstraction
boundaries in the runtime modules.

## Scheduler Architecture

There are two scheduler families for deliberate reasons.

- `ClockScheduler` owns deterministic externally driven clocks.
- `TimingScheduler` owns delayed future slots scheduled by the HDL itself.

They are not merged because the ownership rules and observable semantics are not
the same. A unified surface would hide important distinctions and make failures
harder to reason about.

## Coverage Architecture

Coverage starts as runtime sampling and ends as offline merge and reporting.

The main phases are:

- build typed or manual coverage groups;
- sample them during test execution;
- persist immutable per-test artifacts;
- merge compatible artifacts offline;
- render text, HTML, and machine-readable outputs.

Fingerprints and provenance are part of the architecture, not optional extras.
They protect against invalid merges and make CI outputs explainable.

## Macro Architecture

`vvm-macros` owns derive and attribute expansion.

Its responsibilities are:

- parse `#[vvm(...)]` inputs;
- expand `Drive`, `Sample`, `Clock`, and `Coverage` derives;
- expand `#[vvm::test]` into normal Rust test registration;
- emit compile-time diagnostics through `syn::Error`-style reporting.

The trybuild suite is the public contract for many macro diagnostics and should
be updated deliberately.

## FFI And Safety

The generated bridge narrows the unsafe surface; it does not make FFI concerns
disappear.

Important invariants:

- native model ownership stays inside the generated wrapper and adapter layers;
- C++ exceptions must not cross the CXX boundary;
- generated ABI details are not a supported public interface;
- tracing resources have an explicit lifetime;
- wrappers are not assumed to be thread-safe by default.

When touching this area, prefer explicit ownership and finalization over clever
implicit cleanup.

## Errors And Diagnostics

VVM keeps error ownership close to the subsystem that produced the problem.

That means contributors should think in domain errors, not in one giant shared
enum. Build errors, timing errors, coverage errors, and runtime mismatches each
keep their own context and source chains.

See the maintainer policy document for the detailed error inventory: `docs/dev/errors-and-diagnostics.md`.

