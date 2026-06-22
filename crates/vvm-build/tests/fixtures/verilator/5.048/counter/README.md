# Counter Verilator metadata fixture

Generated with:
- Verilator 5.048

From:
- `counter.sv`

Using:
```bash
verilator \
    --json-only \
    --json-only-output counter.tree.json \
    --json-only-meta-output counter.tree.meta.json \
    --no-json-edit-nums \
    --top-module counter \
    counter.sv
```
