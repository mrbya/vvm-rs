# Multi-clock example

The core clock is the primary transaction clock: it drives sequence boundaries, sampling, model prediction, scoreboard checks, and cycle counts. The peripheral clock transitions independently and evaluates the DUT at every edge without consuming a sequence item.

At a same-time primary inactive boundary, all due clocks transition, the next stimulus is driven, then the DUT evaluates once. At a primary active boundary, evaluation precedes sampling and checking.

Time | Core | Peripheral | Action
---- | ---- | ---------- | ------
0 | inactive | inactive | first stimulus, evaluate
1 | | active | evaluate
2 | | inactive | evaluate
3 | | active | evaluate
4 | active | inactive | evaluate, sample, check
5 | inactive | active | next stimulus, evaluate

Run with `cargo test -p vvm-example-multi-clock`.
