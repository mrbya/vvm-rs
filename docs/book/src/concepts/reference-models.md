# Reference Models

A VVM reference model predicts what the DUT should do for a given input history.

It should model the external contract of the design, not copy the RTL
implementation mechanically. A good reference model answers questions like:

- what value should appear next;
- whether a push or pop should be accepted;
- which flags should be visible after the edge;
- what protocol frame should have been transmitted.

The model can be stateful. That is normal. The synchronous FIFO example uses a
`VecDeque` because the contract is queue ordering, not internal pointer math.
The timed UART example reconstructs frames from serial output levels because the
contract is protocol correctness, not HDL statement-by-statement identity.
