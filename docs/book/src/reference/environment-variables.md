# Environment Variables

| Name | Format | Default | Precedence | Affects | Error behavior | Example |
| --- | --- | --- | --- | --- | --- | --- |
| `VVM_REPLAY` | replay token string | unset | higher than `VVM_SEED` | replay-capable tests | parse failure is reported by config parsing | `VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test counter_random` |
| `VVM_SEED` | integer seed | unset | below `VVM_REPLAY` | seeded randomized tests | parse failure is reported by config parsing | `VVM_SEED=0x1234 cargo test fifo_random` |
| `VVM_CYCLES` | unsigned integer | test default | runtime override | cycle-capable tests | parse failure is reported by config parsing | `VVM_CYCLES=1000 cargo test counter_random` |
| `VVM_TRACE_DIR` | writable directory path | `target/vvm-trace/` rooted generated run dir | runtime override | trace-capable tests | directory/write failures surface through trace setup | `VVM_TRACE_DIR=target/traces cargo test counter_smoke` |
| `VVM_COVERAGE_DIR` | writable directory path | `target/vvm-coverage/` rooted generated run dir | runtime override | coverage-capable tests | persistence failures are retained as diagnostics | `VVM_COVERAGE_DIR=target/vvmcov cargo test fifo_random` |
| `VERILATOR` | executable path | `PATH` lookup | build-time override | `vvm-build` | bad path fails the build stage that launches Verilator | `VERILATOR=/opt/verilator/bin/verilator cargo test` |

`VVM_REPLAY` is the most important override for reproducibility because it
reconstructs the exact pseudo-random stream instead of merely choosing a new
seed.
