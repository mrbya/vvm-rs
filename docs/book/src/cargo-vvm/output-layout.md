# Output Layout

The output directory must be dedicated to one run. `cargo-vvm` writes:

- per-test artifacts under an artifacts directory;
- one merged JSON file;
- one text report;
- one HTML report.

This isolation matters because provenance and merge behavior depend on a clean
set of child-run inputs.
