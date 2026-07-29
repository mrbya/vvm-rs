# Example Strategy

User-facing examples teach an end-to-end verification workflow. Fixtures prove
a narrow build, generated-port, or scheduler contract and are never recommended
as a learning path.

## Learning ladder

`counter` is the beginner path; `sync-fifo` introduces a stateful queue model;
`timed-uart` introduces internally scheduled time; `async-fifo` introduces
independent clocks; `tri-state-bus` is the specialist inout workflow.

| Original project | Original purpose | VVM/HDL feature | Final location | Preserved coverage |
| --- | --- | --- | --- | --- |
| counter | Basic vertical workflow | Drive, Sample, clock, coverage | `examples/counter` | Smoke, replay, trace, coverage, failure diagnostics |
| packed-array | Packed dimensions | Generated packed access | `tests/fixtures/native-packed-array` | Native copied-consumer test |
| packed-enum | Enum mapping | Packed enum values | `tests/fixtures/native-packed-enum` | Native copied-consumer test |
| packed-struct | Aggregate mapping | Packed structures | `tests/fixtures/native-packed-struct` | Native copied-consumer test |
| signed-adder | Signed scalar mapping | Exhaustive signed arithmetic | `tests/fixtures/native-signed-adder` | Native copied-consumer test |
| unpacked-array | Array mapping | Unpacked ports | `tests/fixtures/native-unpacked-array` | Native copied-consumer test |
| wide-transform | Wide packed values | `Bits` and signed bits | `tests/fixtures/native-wide-transform` | Native copied-consumer test |
| timing-delay | Delay scheduler probe | Timed DUT events | `tests/fixtures/native-timing-delay` | Scheduler, trace, finalization tests |
| multi-clock | Synthetic scheduler probe | Independent clocks | `tests/fixtures/native-multi-clock-counter` | Focused native same-order clock regression |
| tri-state-bus | Inout workflow | Resolution and settling | `examples/tri-state-bus` | Native inout tests and VCD |

Every public example needs a README with a purpose, ASCII block diagram,
interface or timing rules, verification goals, public VVM features, layout,
commands, expected result, tracing/replay/coverage guidance where relevant,
limitations, and a next step. Example code must use supported public APIs and
keep reference models independent of RTL implementation details.

`just test-examples` executes the public ladder. `just test-native-fixtures`
copies every native fixture, rewrites only public dependency placeholders, and
runs it with isolated target, trace, and coverage locations. CI invokes both
through `just test-native`.

The counter, synchronous FIFO, and asynchronous FIFO additionally run `cargo vvm
coverage` through their contributor workflows, preserving per-test JSON,
deterministic merge, text report, HTML report, and the GitLab metric.

## Bus-peripheral evaluation

An APB or Wishbone peripheral was evaluated and deferred after v0.1.0. The FIFO
already teaches structured transactions and stateful reference behavior. A bus
example would require a deliberate driver and register-access strategy rather
than an accidental pre-release abstraction. A future bus milestone should own
transaction driving, register semantics, protocol coverage, and error behavior.
