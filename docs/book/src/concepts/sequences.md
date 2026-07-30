# Sequences

VVM deliberately uses ordinary Rust iterators for stimulus sequences. That means
you can write:

- fixed directed sequences;
- pseudo-random replayable sequences;
- sequences built from combinators;
- small helper generators in normal Rust modules.

This design keeps VVM aligned with the language and avoids inventing a second
mini-language for sequence generation.

The main practical rule is to keep the iterator contract clear. A sequence is
responsible for choosing transactions, not for evaluating the DUT or checking
results. That separation is what makes replay and debugging manageable.
