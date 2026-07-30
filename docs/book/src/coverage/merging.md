# Merging

Coverage merging is deterministic and offline.

Important merge concepts:

- merge policy decides which test statuses contribute;
- fingerprints prevent structurally incompatible definitions from being merged;
- provenance stays available for debugging and auditability.

This design keeps the runtime simple and avoids hidden shared-state updates
during test execution.
