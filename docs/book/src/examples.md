# Examples

The maintained learning ladder is Counter, Synchronous FIFO, Timed UART or Asynchronous FIFO, then Tri-state bus. Run all examples with `just test-examples`.

## Counter

The [counter](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/counter) introduces generated inclusion, typed drive/sample/clock derives, a sequence, model, scoreboard, replay, VCD tracing, and functional coverage. Inspect its emitted coverage artifacts after `just functional-coverage-example`. Next: synchronous FIFO.

## Synchronous FIFO

The [synchronous FIFO](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/sync-fifo) models queue ordering and boundary behavior with a `VecDeque` reference model and randomized transactions. Next: timing or independent clocks.

## Timed UART

The [timed UART](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/timed-uart) demonstrates HDL delays, `TimingScheduler`, timestamped protocol reconstruction, parity diagnostics, and tracing. Next: asynchronous FIFO or tri-state bus.

## Asynchronous FIFO

The [asynchronous FIFO](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/async-fifo) demonstrates independently scheduled clocks and deterministic same-time ordering. Next: tri-state bus.

## Tri-state bus

The [tri-state bus](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/tri-state-bus) demonstrates raw inout components, caller-owned resolution, explicit floating and contention policy, bounded settling, and VCD tracing.
