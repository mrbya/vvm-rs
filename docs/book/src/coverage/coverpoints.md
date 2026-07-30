# Coverpoints

A coverpoint samples one semantic dimension.

Common choices include:

- the operation being attempted;
- a boundary state such as empty, available, or full;
- a decoded protocol mode;
- a grouped output range.

Illegal bins take precedence over ignore bins, and ignore bins take precedence
over normal bins. That ordering matters when a value could match more than one
category.

Use a coverpoint when one question is worth asking independently. Use a cross
when the interaction between questions matters.
