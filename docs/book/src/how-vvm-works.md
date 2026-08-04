# How VVM Works

Once your DUT wrapper has been generated and your test starts, VVM runs a repeat
able verification cycle around the DUT.

## Runtime Pipeline

```text
Sequence
   |
   v
Stimulus transaction
   |
   v
Drive inputs
   |
   v
Evaluate DUT and apply clock transitions
   |
   v
Sample outputs
   |
   |-------------> Functional coverage
   v
Reference-model prediction
   |
   v
Scoreboard comparison
   |
   v
Structured result and diagnostics
```

## The Main Pieces

- DUT: the generated Rust wrapper around the Verilator model.
- Stimulus transaction: one semantic input step, such as "write this value while
  enable is high".
- Observation: the subset of DUT outputs you care about after evaluation.
- Sequence: the ordered stream of stimulus transactions.
- Clock: the policy that applies clock transitions for cycle-driven tests.
- Reference model: the behavioural prediction of what the DUT should do.
- Scoreboard: the component that compares prediction and observation.
- Coverage model: optional capture of which meaningful behaviours were exercised.
- Test result: the structured outcome, including failures, replay information,
  trace references, and final statistics.

## Phase By Phase

### 1. Sequence

The sequence decides which stimulus transaction comes next. In simple tests it
can just be an iterator over a fixed array. In randomized tests it may be a
deterministic pseudo-random stream.

### 2. Drive

VVM writes the current stimulus transaction onto the DUT inputs. This phase does
not itself decide whether the DUT state should advance.

### 3. Evaluate And Clock

For ordinary cycle-driven tests, VVM applies the configured clock behaviour and
evaluates the DUT. This is the point where sequential HDL state updates become
visible.

### 4. Sample

After evaluation, VVM reads the outputs you selected into an observation value.
Sampling records what the DUT did; it does not decide whether that behaviour was
correct.

### 5. Predict

The reference model consumes the same stimulus transaction and predicts the
expected observation.

### 6. Compare

The scoreboard compares expected and observed behaviour. If they differ, VVM
retains a structured failure record instead of leaving you to reverse-engineer a
raw assertion message.

### 7. Cover

If you attached a coverage model, VVM can sample the observed cycle into that
model so you can track which meaningful behaviours were exercised.

### 8. Finalize

At the end of the run, VVM finalizes the result, closes any open trace, and
returns the structured outcome to the test harness.

Read [Rust Essentials For HDL Engineers](rust-essentials-for-hdl-engineers.md)
next if the Rust-specific terms in this chapter are unfamiliar.
