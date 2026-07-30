# Crosses

Cross coverage answers questions about combinations, not individual dimensions.

In VVM, `Cross2` combines the normal-bin identities of two successful
coverpoint samples. Ignored or unmatched axes do not produce a cross hit.

That makes crosses useful for questions like:

- which operation happened at which occupancy boundary;
- which parity mode was used with which injected error class.

Crosses should stay meaningful. If a cross has many bins but no clear
verification value, it is noise rather than insight.
