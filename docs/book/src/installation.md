# Installation

## Requirements

For normal VVM usage, install:

- Rust 1.87.0 or newer.
- The current stable Rust toolchain is also validated in CI.
- Cargo.
- Verilator.
- A working Linux C++ toolchain.

Ordinary native models use a C++17-capable toolchain.

Timing-enabled models additionally need a C++20 compiler with coroutine support.

VVM does not currently claim broad cross-platform native support. The tested
path is Linux-based development and CI, with Verilator 5.000 as the minimum
supported version and 5.050 as the current validated image.

## Add Dependencies To A Consumer Project

Most users depend on `vvm-rs` as `vvm` in test code and `vvm-build` in
`build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.2.0" }

[build-dependencies]
vvm-build = "0.2.0"
```

Until `v0.2.0` is published, use a Git dependency when you need the unreleased
main-branch development state:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", git = "https://gitlab.com/byacrates/vvm-rs.git" }

[build-dependencies]
vvm-build = { git = "https://gitlab.com/byacrates/vvm-rs.git" }
```

Install `cargo-vvm` separately when you want suite-level functional-coverage
merge and reports:

```bash
cargo install cargo-vvm
```

## Contributor Setup

The repository uses `just` recipes as the primary command surface. Install
`just`, then bootstrap the workspace once:

```bash
cargo install just
just init
```

`just init` installs the extra tooling used by this repository, including
nightly rustfmt, nextest, LLVM coverage, udeps, audit, mdBook, Markdown TOC,
and pre-commit hooks.

## Verify Your Toolchain

Useful checks before starting a new VVM project:

```bash
rustc --version
cargo --version
verilator --version
```

If Verilator is not on `PATH`, VVM can use an explicit executable path through
the `VERILATOR` environment variable during the build.

## Next Steps

- Follow [Quick Start](quick-start.md) for a complete first project.
- Use [Project Setup](guide/project-setup.md) when adapting the workflow to an
  existing repository.
