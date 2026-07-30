# Coverage Architecture

Coverage starts as runtime sampling and ends as offline merge and reporting.

The main phases are:

- build typed or manual coverage groups;
- sample them during test execution;
- persist immutable per-test artifacts;
- merge compatible artifacts offline;
- render text, HTML, and machine-readable outputs.

Fingerprints and provenance are part of the architecture, not optional extras.
They protect against invalid merges and make CI outputs explainable.
