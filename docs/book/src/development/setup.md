# Setup

Repository setup is intentionally simple:

```bash
cargo install just
just init
```

The bootstrap installs the tooling used by the repository's validation commands
and pre-commit hooks. The `justfile` remains the source of truth for exact
command behavior.
