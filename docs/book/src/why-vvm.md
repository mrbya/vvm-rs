# Why VVM?

VVM exists for teams that want a compiled, typed verification workflow around
Verilator without owning the usual C++ integration layer themselves.

## The Problem

Many HDL teams already know how to write testbenches, but they still face one of
these trade-offs:

| Approach | Strengths | Trade-offs |
| --- | --- | --- |
| Commercial simulators and verification suites | Mature simulators, four-state behaviour, broad language support, established debug tooling, commercial support | Licence cost, large tool surfaces, closed ecosystems, and workflows that often stay HDL- or UVM-centric |
| Free vendor tools | No licence cost for supported devices, integration with implementation flows, device-specific support | Closed infrastructure, vendor lock-in, heavy setup, and verification flows still centred on HDL-specific tooling |
| cocotb and Python-based verification | Accessible language, broad simulator compatibility, fast test authoring, large ecosystem | Python-centric runtime, dynamic typing, and integration/performance characteristics that differ from native compiled workflows |
| Raw Verilator with C++ | Free and open source, fast compiled simulation, direct control | Manual wrapper code, repeated infrastructure, and user-owned lifecycle, tracing, error handling, and reporting |

## What VVM Tries To Combine

VVM aims to combine:

- Verilator's compiled simulation model.
- Rust's type system and memory-safety model.
- Ordinary Cargo build and test workflows.
- Generated typed DUT wrappers.
- Reusable drive, sample, clock, sequence, model, scoreboard, and coverage
  patterns.
- Deterministic replay, tracing, and structured diagnostics.

The goal is not to prove that Rust is universally easier than C++.

The goal is narrow and more practical: most users should not need to hand-own
convoluted testbench lifecycle boilerplate, repeated trace plumbing, or repetitive DUT
access scaffolding just to verify a design. VVM aims to provide it in a safe, strongly typed
language with a mature ecosystem.

## What VVM Does Not Try To Replace

VVM does not try to replace every verification tool.

- It is not a four-state simulator.
- It is not a UVM compatibility layer.
- It is not a formal CDC or metastability tool.
- It does not remove the value of commercial simulators when you need their
  language support, debug tooling, or support contracts.

## When VVM Is A Good Fit

VVM is a good fit when you want:

- open-source compiled simulation based on Verilator;
- strongly typed transactions and observations;
- deterministic replay for pseudo-random regressions;
- reusable Rust-side models, scoreboards, and coverage code;
- a workflow that can scale from a smoke test to a larger verification harness.

## When Another Tool Is Better

Another tool is often better when you need:

- production-proven four-state behaviour or broader HDL language support;
- vendor-specific simulation features tied to a device flow;
- an existing team workflow already standardized on another environment;

## Current Position

VVM is still alpha software. The project already supports real end-to-end
verification workflows, but the public API and supported feature surface are not
yet frozen.

Read [How VVM Fits Into HDL Verification](how-vvm-fits-into-hdl-verification.md)
next for the build-time and test-time pipeline.
