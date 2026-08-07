#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(git rev-parse --show-toplevel)"
cd "$workspace_root"

publishable_packages=(vvm-core vvm-macros vvm-build vvm-rs cargo-vvm)
release_tag="${CI_COMMIT_TAG:-${1:-}}"
protected_ref="${CI_COMMIT_REF_PROTECTED:-false}"
version="$(awk '
  $0 == "[workspace.package]" { in_workspace = 1; next }
  in_workspace && /^\[/ { exit }
  in_workspace && $1 == "version" {
    gsub(/"/, "", $3)
    print $3
    exit
  }
' Cargo.toml)"

if [[ "${VVM_RELEASE_PUBLISH:-0}" != "1" ]]; then
  printf 'refusing to publish without VVM_RELEASE_PUBLISH=1\n' >&2
  exit 1
fi

if [[ -z "$release_tag" ]]; then
  printf 'refusing to publish without a release tag\n' >&2
  exit 1
fi

if [[ "$protected_ref" != "true" ]]; then
  printf 'refusing to publish from an unprotected ref\n' >&2
  exit 1
fi

if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  printf 'refusing to publish without CARGO_REGISTRY_TOKEN\n' >&2
  exit 1
fi

if [[ "$version" == *-dev* ]]; then
  printf 'refusing to publish development version %s\n' "$version" >&2
  exit 1
fi

expected_tag="v${version}"

if [[ "$release_tag" != "$expected_tag" ]]; then
  printf 'release tag mismatch: expected %s but received %s\n' "$expected_tag" "$release_tag" >&2
  exit 1
fi

bash scripts/release-verify.sh "$release_tag"

for package in "${publishable_packages[@]}"; do
  cargo publish -p "$package"
done
