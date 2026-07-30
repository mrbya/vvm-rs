# Scoreboards

A scoreboard compares the predicted result with the observed result.

`ExactScoreboard` is the common starting point: it checks exact equality and
returns a `Mismatch` when the observed output differs from the prediction.

The important design point is that the scoreboard owns comparison policy. That
lets you separate:

- what traffic was generated;
- what behavior was expected;
- what counts as an error.

Failures carry cycle-aware context so the final report can say which cycle or
time slot failed, not just that something failed somewhere in the run.
