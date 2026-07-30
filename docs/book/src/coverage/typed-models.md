# Typed Models

`#[derive(Coverage)]` lets you collect related coverpoints and crosses into one
typed model with a stable definition name, revision, and instance path.

The counter example is the easiest starting point because it shows:

- a semantic operation coverpoint;
- grouped output regions;
- parity as a second observation dimension;
- crosses over meaningful pairs.

Use a typed model when the coverage structure belongs to one reusable design or
subsystem. Use manual group construction when the coverage logic is more ad hoc,
as in the timed UART and asynchronous FIFO examples.
