# Artifact Layout

Common artifact roots:

- traces under `target/vvm-trace/` by default;
- coverage artifacts under `target/vvm-coverage/` by default;
- assembled documentation site under `public/`;
- mdBook build output under `target/book/`.

`cargo-vvm` writes an isolated output tree containing per-test artifacts,
merged JSON, text, and HTML outputs.
