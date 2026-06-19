# VVM

> Verilator Verification Methodology

<!-- toc -->

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Development](#development)
  * [Prequisites](#prequisites)
  * [Getting started](#getting-started)
- [Documentation](#documentation)
  * [Style](#style)
- [License](#license)

<!-- tocstop -->

## Features

TBD

---

## Installation

Install using cargo:
```bash
cargo install vvm
```

## Usage

TBD

---

## Development

### Prequisites

- Rust stable toolchain with `rustfmt` and `clippy` (`rust-toolchain.toml`)
- Rust `1.85.0` or newer for workspace builds
- [`just`](https://crates.io/crates/just)

Rest TBD

For a first time setup, run:
```bash
cargo install just
just init
```

This installs just and its `init` bootstrap recipe installs all extra tooling used by this repository, including coverage, lint/audit tools, README indexing, pre-commit hooks and more.

### Getting started

TBD

---

## Documentation

TBD

### Style

Codebase documented using a consistent rustdoc style described in [rustdoc style guide](docs/rustdoc_style.md).

---

## License

Dual licensed under:

- Apache License 2.0 (`LICENSE-APACHE`)
- MIT (`LICENSE-MIT`)
