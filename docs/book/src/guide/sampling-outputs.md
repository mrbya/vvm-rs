# Sampling Outputs

`Sample` reads generated DUT outputs into a Rust observation type.

Like `Drive`, it has a narrow job:

- it reads outputs;
- it does not evaluate the DUT;
- it does not predict expected behavior;
- it does not decide pass or fail.

Observation types should represent what you want to compare. That may be a raw
port value, or it may be a more semantic shape if a manual `Sample`
implementation makes the result clearer, as in the synchronous FIFO example.
