# Failures And Results

VVM keeps pass/fail information structured instead of flattening everything into
one string early.

Important result concepts include:

- simulation failures from DUT access or execution stages;
- scoreboard mismatches with expected and observed values;
- retained coverage diagnostics;
- replay tokens for randomized tests;
- detailed reports for human investigation.

Failure policy affects how much of a run is retained. Stop-on-first-failure is
often best for short tests. Collected failures are useful when one run should
show several mismatches before stopping.

Coverage is related but separate: a test can fail and still leave behind useful
coverage information about what it managed to execute before stopping.
