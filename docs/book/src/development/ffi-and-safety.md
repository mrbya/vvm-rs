# FFI And Safety

The generated bridge narrows the unsafe surface; it does not make FFI concerns
disappear.

Important invariants:

- native model ownership stays inside the generated wrapper and adapter layers;
- C++ exceptions must not cross the CXX boundary;
- generated ABI details are not a supported public interface;
- tracing resources have an explicit lifetime;
- wrappers are not assumed to be thread-safe by default.

When touching this area, prefer explicit ownership and finalization over clever
implicit cleanup.
