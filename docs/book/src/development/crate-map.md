# Crate Map

| Crate | Responsibility | What belongs here | What should not live here |
| --- | --- | --- | --- |
| `vvm-rs` | public facade and exports | user-facing modules, derives, docs-facing paths | core implementation details |
| `vvm-core` | runtime primitives | testbench, timing, packed values, coverage internals | facade ergonomics and build-script logic |
| `vvm-build` | build-time generation | Verilator execution, metadata parsing, code generation | runtime verification policy |
| `vvm-macros` | proc macros | derive parsing, expansion, diagnostics | runtime data structures |
| `cargo-vvm` | binary-only orchestration | CLI parsing, child command orchestration, report writing | public library API contracts |

Examples and fixtures prove consumer workflows and code-generation contracts; do
not treat them as dumping grounds for production logic that belongs in a crate.
