# Sessions And Artifacts

Coverage capture is test-scoped. Each captured run writes one immutable artifact
that records the sampled groups, provenance, and definition fingerprint.

This separation matters because merging is intentionally offline. One test does
not mutate another test's live runtime state.

When a test captures no coverage, VVM does not fabricate an empty artifact just
to make downstream tooling look successful.
