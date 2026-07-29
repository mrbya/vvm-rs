# Contributing

Run `just init` once to install nightly formatting and documentation tooling, nextest, LLVM coverage, udeps, audit, Markdown TOC, and pre-commit. Primary commands are `just fmt --check`, `just check -- -D warnings`, `just test-fast`, `just test-native`, `just test-examples`, and `just ci`.

Use curated examples for user workflows and isolated fixtures for narrow generated-port regressions. Preserve fixture golden data and the test-category boundaries described in the [testing strategy](../../dev/testing-strategy.md). Documentation changes must keep rustdoc contracts precise, use tested example sources for generated-DUT snippets, and run `just docs-test` and `just docs-links`.

The documentation site is built by `just docs-site`; `just docs-serve` serves it locally. `just docs-internal` is maintainer-only private-item rustdoc and is never deployed. External link checks are intentionally scheduled or default-branch work because they depend on remote availability.

Contributors must retain MSRV 1.87.0, the Linux native-support statement, and the Verilator 5.000 minimum / 5.050 tested claims unless CI evidence and the compatibility documentation change together. Benchmark workflows use `just benchmark`; releases and publication policy remain controlled by the roadmap.
