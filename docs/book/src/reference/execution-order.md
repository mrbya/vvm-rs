# Execution Order

The observable cycle-driven order is:

1. drive stimulus;
2. evaluate the DUT and apply the configured clock transition(s);
3. sample outputs;
4. predict expected behavior;
5. compare with the scoreboard;
6. record coverage;
7. retain diagnostics and finalize the result.

Timing-enabled execution instead advances from one pending delayed slot to the
next. That path is separate because the DUT, not the Rust testbench, owns the
future event queue.
