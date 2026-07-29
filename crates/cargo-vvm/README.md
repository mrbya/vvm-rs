# cargo-vvm

`cargo-vvm` orchestrates offline VVM functional-coverage merging and reporting.

```text
cargo install cargo-vvm
cargo vvm coverage --output target/vvm-coverage --name counter -- test -p my-tests
```

It preserves the child test exit status, writes merged JSON plus text and HTML reports when artifacts exist, and reports no-artifact and reporting failures explicitly. See the [cargo-vvm guide](https://byacrates.gitlab.io/vvm-rs/cargo-vvm.html) for output layout, merge policies, bin detail, fingerprints, and GitLab metric integration.
