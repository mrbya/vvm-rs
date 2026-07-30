# Driving Inputs

`Drive` maps a Rust type onto generated DUT input setters.

The derive is intentionally narrow in responsibility:

- it writes inputs;
- it does not evaluate the DUT;
- it does not toggle clocks;
- it does not sample outputs.

That separation makes testbench phases predictable. A transaction type should
normally include only the ports that belong to one meaningful input step.

Use higher-level semantic fields where practical. The synchronous FIFO example is
easier to understand because it talks about push/pop requests, not about a raw
mirror of every internal queue signal.
