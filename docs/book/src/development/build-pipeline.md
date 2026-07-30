# Build Pipeline

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
