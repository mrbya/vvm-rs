# Diagnostics

VVM diagnostics are structured around stages and domains.

Look for:

- build stage when Verilator or native code generation fails;
- cycle or simulation time when a mismatch occurs;
- replay token when a randomized test failed;
- persistence, merge, or report context for coverage failures.

This structure is why the project keeps domain-specific error types instead of
one giant user-visible error enum.
