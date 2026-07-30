# Command Reference

The binary currently exposes one top-level workflow:

```text
cargo vvm coverage [OPTIONS] [-- CHILD-CARGO-COMMAND]
```

The command after `--` excludes the leading `cargo`. Supported child forms are
the normal `test ...` path and `nextest run ...`.

Common options:

- `--output`
- `--name`
- `--merge-policy`
- `--bin-detail`
- `--no-inputs`
- `--fingerprints`

If no child command is supplied, the tool falls back to its built-in default.
