# Transactions

A transaction is the typed unit of meaning you want to apply or observe.

In a counter, a transaction might be just `reset_n` and `enable`. In a FIFO, it
is usually a higher-level push/pop request instead of raw internal flags. In a
bus-oriented example, it may need request and response types.

Good VVM transaction types usually follow these rules:

- They describe behavior at the boundary you care about.
- They hide irrelevant DUT internals.
- They are cheap to clone or copy when practical.
- They derive `Drive` or `Sample` only for the fields that map to ports.

The goal is not to mirror the RTL implementation line by line. The goal is to
create types that keep the verification flow readable.
