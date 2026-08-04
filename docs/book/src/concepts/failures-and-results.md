# Failures And Results

VVM keeps pass/fail information structured instead of flattening everything into
one string early.

## Where It Fits

Failures and results are the output of the whole verification pipeline.

```text
Simulation failure
        |
        +--> retained runtime diagnostic

Scoreboard mismatch
        |
        +--> retained check failure

Coverage sampling
        |
        +--> retained artifact/report data
```

Important result concepts include:

- simulation failures from DUT access or execution stages;
- scoreboard mismatches with expected and observed values;
- retained coverage diagnostics;
- replay tokens for randomized tests;
- detailed reports for human investigation.

## Why Structured Results Exist

Failure policy affects how much of a run is retained. Stop-on-first-failure is
often best for short tests. Collected failures are useful when one run should
show several mismatches before stopping.

Coverage is related but separate: a test can fail and still leave behind useful
coverage information about what it managed to execute before stopping.

## What Results Own

- the retained failures from a run;
- cycle or time-slot context for those failures;
- replay and coverage artifacts that help you reproduce or inspect the run.

## What Results Do Not Own

- stimulus generation;
- DUT port access;
- comparison policy;
- coverage-definition design.

## Related Material

- [Configuring Tests](../guide/configuring-tests.md)
- [Troubleshooting](../guide/troubleshooting.md)
- [Errors API Guide](../api-guide/errors.md)
