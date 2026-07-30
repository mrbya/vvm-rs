# Bins

Bins are the basic unit of functional coverage intent.

VVM supports:

- normal bins for values you want to count;
- ignore bins for values you want to classify but exclude from coverage goals;
- illegal bins for values that should be called out as invalid.

Bins can match:

- exact values;
- explicit value sets;
- inclusive ranges.

Good bins reflect verification intent. Do not create bins only because an API
allows them. The counter example uses regions such as `zero`, `low`, `medium`,
`high`, and `maximum` because those are meaningful observation classes.
