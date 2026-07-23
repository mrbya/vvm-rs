# Coverage JSON Schema v1

VVM writes one functional-coverage artifact for each VVM test that captures at
least one coverage group through `TestContext::capture_coverage`, unless
persistence fails. The standard test bridge attempts the write after the test
body has run, including for a failed or error-status test. It does not write an
artifact for a test with no captured coverage.

Artifacts use the `.vvmcov.json` suffix. Set `VVM_COVERAGE_DIR` to choose their
root directory:

```console
VVM_COVERAGE_DIR=target/coverage-artifacts cargo test decoder_random
```

The bridge creates a filename of the form
`pid-<process-id>-<test-name>.vvmcov.json`. If the variable is unset, the root
is a generated per-run directory below `target/vvm-coverage/`. The configured
root must not be empty and, when it already exists, must be a directory.

## Top-Level Document

Schema v1 is a JSON object with these required fields:

| Field | Type | Meaning |
|---|---|---|
| `format` | string | Always `vvm-functional-coverage`. |
| `schema_version` | integer | Always `1`. |
| `producer` | object | Producer `name` (`vvm-rs`) and non-empty package `version`. |
| `test` | object | Test `name`, `status`, and optional `replay_token`. |
| `summary` | object | Recomputed aggregate counts and exact coverage ratio. |
| `groups` | array | Captured groups in capture order; at least one is required. |

`test.status` is one of `passed`, `failed`, or `error`. `replay_token` is a
string when available and `null` otherwise.

Coverage ratios have `covered`, `uncovered`, and `total` integer fields. They
are exact bin counts, and `covered + uncovered == total`; v1 does not store a
percentage.

## Groups And Items

Each group has `definition`, `instance_path`, `summary`, and `items` fields.
`definition` contains the group definition `name`, semantic `revision`, and
`fingerprint`. The group summary contains item, coverpoint, and cross counts
plus an exact ratio.

Items are tagged by `kind`:

| Kind | Additional fields |
|---|---|
| `coverpoint` | Sample classifications, exact coverage ratio, and ordered bins. |
| `cross2` | Source coverpoint names, sample and skipped-sample counts, exact coverage ratio, and ordered cross bins. |

`coverpoint` items contain `name`, `samples`, `ignored_samples`,
`illegal_samples`, `unmatched_samples`, `coverage`, and `bins`. Each bin
records its declaration `id`, `name`, `kind`, matcher shape (`kind` and
`operand_count`), `hits`, and `required_hits`.

`cross2` items contain `name`, `left_coverpoint`, `right_coverpoint`, `samples`,
`skipped_samples`, `coverage`, and `bins`. Each cross bin records its row-major
`id`, `left_bin_id`, `left_bin_name`, `right_bin_id`, `right_bin_name`, `hits`,
and `required_hits`.

The reader rejects unknown fields and validates all stored summaries, ratios,
identifiers, bin ordering, cross-source relationships, and fingerprints against
the reconstructed document.

## Definition Fingerprints

Each group fingerprint is formatted as `sha256-v1:<64 lowercase hex digits>`.
It is SHA-256 over a versioned canonical binary encoding of the group
definition: definition name and revision, item order and names, bin IDs, names,
kinds, required-hit thresholds, matcher kinds and operand counts, and cross
source and generated-bin structure.

The fingerprint intentionally does not include sample counters, bin hit counts,
or the group instance path. It also does not include matcher operand values or
inclusive-range bounds: schema v1 persists matcher shape, not matcher values.
Therefore a matching v1 fingerprint is not proof that two independently built
coverage definitions have identical matcher values, and it is not a merge
compatibility guarantee. Deterministic artifact merging is not implemented.

## Writing

`CoverageArtifact::write_to` serializes pretty JSON with one trailing newline,
creates missing parent directories, and refuses a destination that already
exists. It writes to a create-new temporary file in the destination directory,
synchronizes that file, then renames it to the final path. On a write failure it
attempts to remove the temporary file.

The temporary file and destination are in the same directory so the final
rename uses the filesystem's same-directory rename behavior. VVM does not
synchronize the parent directory, and concurrent writers targeting the same
destination are unsupported. Use the bridge-generated per-process test names
or otherwise choose distinct destination paths.
