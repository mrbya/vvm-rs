# Getting Started

Use this page if you are starting from the long-lived `getting-started.html`
entry point and want the shortest path through the book.

## Who This Path Is For

This path is for readers who already understand digital design and simulation,
but may be new to Rust, Cargo, or the VVM-specific workflow.

## Recommended Reading Order

If you are new to VVM, read these chapters in order:

1. [Introduction](introduction.md) for purpose, scope, and limitations.
2. [Why VVM?](why-vvm.md) for tool-positioning and trade-offs.
3. [How VVM Fits Into HDL Verification](how-vvm-fits-into-hdl-verification.md)
   for the build pipeline.
4. [How VVM Works](how-vvm-works.md) for the runtime model.
5. [Rust Essentials For HDL Engineers](rust-essentials-for-hdl-engineers.md)
   for the minimum Rust context.
6. [Installation](installation.md) for prerequisites and dependency setup.
7. [Quick Start](quick-start.md) for a complete first VVM test.

## What You Will Build

The Quick Start creates a small independent verification crate from scratch. It
shows the HDL module, `build.rs`, generated wrapper inclusion, typed stimulus and
observation, a deterministic sequence, a reference model, a scoreboard, and a
registered VVM test.

After that, use the [Guide](user-guide.md) for task-oriented instruction and the
[Examples](examples.md) section for larger case studies.
