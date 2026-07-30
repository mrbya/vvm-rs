# Errors And Diagnostics

VVM keeps error ownership close to the subsystem that produced the problem.

That means contributors should think in domain errors, not in one giant shared
enum. Build errors, timing errors, coverage errors, and runtime mismatches each
keep their own context and source chains.

See the maintainer policy document for the detailed error inventory:
[`docs/dev/errors-and-diagnostics.md`](../../../dev/errors-and-diagnostics.md)
