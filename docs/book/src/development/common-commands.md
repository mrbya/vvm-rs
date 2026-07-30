# Common Commands

| Command | Scope |
| --- | --- |
| `just fmt --check` | formatting only |
| `just check -- -D warnings` | clippy across the workspace |
| `just test-unit` | library unit targets |
| `just test-integration` | public pure-Rust crate contracts |
| `just test-ui` | macro trybuild suites |
| `just test-fixtures` | clean pure consumer fixtures |
| `just test-examples` | curated native examples |
| `just test-native-fixtures` | copied native HDL fixtures |
| `just test-fast` | non-native suites |
| `just test-native` | native suites |
| `just test-e2e` | release-facing native workflows |
| `just test-package` | package archive verification |
| `just test-all` | full test matrix |
| `just docs-site` | assemble book and public rustdoc |
| `just ci` | full repository gate |

Prefer `just` recipes over ad hoc Cargo commands whenever a recipe exists.
