# Merge Policies

Merge policy decides which child test results contribute artifacts to the final
merge.

The stable CLI spellings are:

- passed-only
- passed-and-failed
- all

Use the narrowest policy that matches the question you are asking. Passed-only is
usually the right default for suite-level reporting.
