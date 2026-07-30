# Creating A Testbench

`Testbench` composes the normal cycle-driven workflow.

The common build order is:

1. `Testbench::new(dut)`;
2. attach a sequence or replayable sequence;
3. attach a reference model;
4. attach a scoreboard;
5. attach a clock;
6. optionally attach coverage;
7. run it.

The runner then owns the cycle lifecycle:

- drive inputs;
- evaluate the DUT and clock transitions;
- sample outputs;
- predict expected behavior;
- compare through the scoreboard;
- record coverage;
- return a structured result.

The counter example is the smallest complete version of this pattern. The FIFO
examples show why keeping the model and scoreboard separate pays off quickly.
