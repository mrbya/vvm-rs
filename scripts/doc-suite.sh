#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

write_redirect() {
    local output_path="$1"
    local destination="$2"

    mkdir -p "$(dirname "$output_path")"
    cat > "$output_path" <<EOF
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="0; url=${destination}">
  <link rel="canonical" href="${destination}">
  <title>VVM documentation moved</title>
</head>
<body>
  <p>This page moved to <a href="${destination}">${destination}</a>.</p>
</body>
</html>
EOF
}

rm -rf public target/docs-api
mdbook build docs/book
mkdir -p public/api
cp -a target/book/. public/
RUSTDOCFLAGS="-D warnings --default-theme ayu" cargo doc --locked --no-deps --all-features -p vvm-rs -p vvm-build -p vvm-core -p vvm-macros --target-dir target/docs-api
cp -a target/docs-api/doc/. public/api/
printf '%s\n' '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>VVM API reference</title></head><body><h1>VVM API reference</h1><ul><li><a href="vvm/">vvm</a></li><li><a href="vvm_build/">vvm-build</a></li><li><a href="vvm_core/">vvm-core</a></li><li><a href="vvm_macros/">vvm-macros</a></li></ul><p><a href="../">Project book</a></p></body></html>' > public/api/index.html

package_id="$(cargo pkgid -p vvm-rs)"
version="${package_id##*#}"
revision="$(git rev-parse HEAD)"
channel="${CI_COMMIT_TAG:-${CI_COMMIT_BRANCH:-local}}"
rust="$(rustc --version)"
timestamp="$(git log -1 --format=%cI HEAD)"
printf '{"version":"%s","revision":"%s","channel":"%s","built_at":"%s","rust":"%s","msrv":"1.87.0","verilator_minimum":"5.000","verilator_tested":"5.050"}\n' "$version" "$revision" "$channel" "$timestamp" "$rust" > public/build-info.json

write_redirect public/getting-started.html quick-start.html
write_redirect public/user-guide.html guide/project-setup.html
write_redirect public/coverage.html guide/functional-coverage.html
write_redirect public/coverage/bins.html ../guide/functional-coverage.html#bins
write_redirect public/coverage/coverpoints.html ../guide/functional-coverage.html#coverpoints
write_redirect public/coverage/typed-models.html ../guide/functional-coverage.html#typed-coverage-models
write_redirect public/coverage/crosses.html ../guide/functional-coverage.html#cross-coverage
write_redirect public/coverage/sampling.html ../guide/functional-coverage.html#sampling-and-observedcycle
write_redirect public/coverage/sessions-and-artifacts.html ../guide/functional-coverage.html#per-test-artifacts-and-snapshots
write_redirect public/coverage/merging.html ../guide/functional-coverage.html#merge-compatibility
write_redirect public/coverage/reporting.html ../guide/functional-coverage.html#reports
write_redirect public/coverage/ci.html ../guide/functional-coverage.html#ci-integration
write_redirect public/cargo-vvm.html guide/using-cargo-vvm.html
write_redirect public/cargo-vvm/installation.html ../guide/using-cargo-vvm.html#installation
write_redirect public/cargo-vvm/workflow.html ../guide/using-cargo-vvm.html#normal-workflow
write_redirect public/cargo-vvm/command-reference.html ../guide/using-cargo-vvm.html#command-summary
write_redirect public/cargo-vvm/output-layout.html ../guide/using-cargo-vvm.html#output-directory-and-layout
write_redirect public/cargo-vvm/merge-policies.html ../guide/using-cargo-vvm.html#merge-policies
write_redirect public/cargo-vvm/failure-semantics.html ../guide/using-cargo-vvm.html#failure-semantics
write_redirect public/cargo-vvm/gitlab.html ../guide/using-cargo-vvm.html#gitlab-ci-example
write_redirect public/cargo-vvm/troubleshooting.html ../guide/using-cargo-vvm.html#troubleshooting
write_redirect public/reference.html guide/configuring-tests.html
write_redirect public/reference/configuration.html ../guide/configuring-tests.html#configuration-layers
write_redirect public/reference/environment-variables.html ../guide/configuring-tests.html#environment-variables
write_redirect public/reference/generated-types.html ../guide/generated-type-mapping.html
write_redirect public/reference/execution-order.html ../guide/execution-order.html
write_redirect public/reference/artifact-layout.html ../guide/using-cargo-vvm.html#output-directory-and-layout
write_redirect public/reference/compatibility.html ../guide/compatibility-and-limitations.html
write_redirect public/reference/diagnostics.html ../guide/troubleshooting.html
write_redirect public/reference/terminology.html ../concepts/verification-workflow.html
write_redirect public/reference/limitations.html ../guide/compatibility-and-limitations.html

test -s public/index.html
test -s public/404.html
test -s public/api/index.html
test -s public/api/vvm/index.html
test -s public/api/vvm_build/index.html
test -s public/api/vvm_core/index.html
test -s public/api/vvm_macros/index.html
test -s public/build-info.json
