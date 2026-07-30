# Runtime

At runtime, VVM owns a clear lifecycle:

1. construct the generated DUT wrapper;
2. initialize any trace or timing state;
3. drive inputs;
4. evaluate the DUT and scheduler-owned transitions;
5. sample outputs;
6. predict and compare;
7. record coverage;
8. finalize and close native resources.

Keeping these phases separate is why the codebase cares about clear abstraction
boundaries in the runtime modules.
