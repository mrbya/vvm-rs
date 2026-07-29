#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

rm -rf public target/docs-api
mdbook build docs/book
mkdir -p public/api
cp -a target/book/. public/
RUSTDOCFLAGS="-D warnings --default-theme ayu" cargo doc --no-deps --all-features -p vvm-rs -p vvm-build -p vvm-core -p vvm-macros --target-dir target/docs-api
cp -a target/docs-api/doc/. public/api/
printf '%s\n' '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>VVM API reference</title></head><body><h1>VVM API reference</h1><ul><li><a href="vvm/">vvm</a></li><li><a href="vvm_build/">vvm-build</a></li><li><a href="vvm_core/">vvm-core</a></li><li><a href="vvm_macros/">vvm-macros</a></li></ul><p><a href="../">Project book</a></p></body></html>' > public/api/index.html

package_id="$(cargo pkgid -p vvm-rs)"
version="${package_id##*#}"
revision="$(git rev-parse HEAD)"
channel="${CI_COMMIT_TAG:-${CI_COMMIT_BRANCH:-local}}"
rust="$(rustc --version)"
timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '{"version":"%s","revision":"%s","channel":"%s","built_at":"%s","rust":"%s","msrv":"1.87.0","verilator_minimum":"5.000","verilator_tested":"5.050"}\n' "$version" "$revision" "$channel" "$timestamp" "$rust" > public/build-info.json

test -s public/index.html
test -s public/404.html
test -s public/api/index.html
test -s public/api/vvm/index.html
test -s public/api/vvm_build/index.html
test -s public/api/vvm_core/index.html
test -s public/api/vvm_macros/index.html
test -s public/build-info.json
