# Generated Types

VVM supports common generated-port mappings such as:

- scalar booleans and integers;
- signed scalar values;
- wide packed values through packed helpers;
- packed arrays, structs, and enums when supported by the generator path;
- unpacked arrays where the generated wrapper exposes indexed access;
- top-level `inout` ports through split state.

The most precise examples live in the maintained fixtures under
`tests/fixtures/native-*` and the tri-state public example.

Unsupported or awkward shapes should be treated as real limitations, not hidden
under vague wording.
