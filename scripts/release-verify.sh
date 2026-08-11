#!/usr/bin/env bash
set -euo pipefail

workspace_root="$(git rev-parse --show-toplevel)"
cd "$workspace_root"

publishable_packages=(vvm-core vvm-macros vvm-build vvm-rs cargo-vvm)
release_tag="${1:-${CI_COMMIT_TAG:-}}"
declare -A extracted_package_roots=()
version="$(awk '
  $0 == "[workspace.package]" { in_workspace = 1; next }
  in_workspace && /^\[/ { exit }
  in_workspace && $1 == "version" {
    gsub(/"/, "", $3)
    print $3
    exit
  }
' Cargo.toml)"

if [[ -z "$version" ]]; then
  printf 'failed to read workspace version from Cargo.toml\n' >&2
  exit 1
fi

expected_tag="v${version}"
release_build_jobs="${CARGO_BUILD_JOBS:-1}"

cargo metadata --locked --no-deps --format-version 1 >/dev/null

if [[ -n "$release_tag" && "$release_tag" != "$expected_tag" ]]; then
  printf 'release tag mismatch: expected %s but received %s\n' "$expected_tag" "$release_tag" >&2
  exit 1
fi

for dependency in vvm-rs vvm-build vvm-core vvm-macros; do
  dependency_key="$dependency"

  if [[ "$dependency" == "vvm-rs" ]]; then
    dependency_key="vvm"
  fi

  dependency_version="$(awk -v dependency_key="$dependency_key" '
    $0 == "[workspace.dependencies]" { in_dependencies = 1; next }
    in_dependencies && /^\[/ { exit }
    in_dependencies && $1 == dependency_key {
      if (match($0, /version *= *"[^"]+"/)) {
        value = substr($0, RSTART, RLENGTH)
        sub(/^version *= *"/, "", value)
        sub(/"$/, "", value)
        print value
        exit
      }
    }
  ' Cargo.toml)"

  if [[ -z "$dependency_version" ]]; then
    printf 'missing workspace dependency declaration for %s\n' "$dependency" >&2
    exit 1
  fi

  if [[ "$dependency_version" != "$version" ]]; then
    printf 'internal dependency %s uses version %s but workspace version is %s\n' \
      "$dependency" \
      "$dependency_version" \
      "$version" >&2
    exit 1
  fi
done

artifact_root="target/vvm-release/${version}"
package_target="${artifact_root}/cargo-target"
docs_artifact="${artifact_root}/docs/book"
package_lists="${artifact_root}/package-lists"
extracted_packages="/tmp/opencode/vvm-release-package-extracted/${version}"

rm -rf "$artifact_root"
rm -rf "$extracted_packages"
mkdir -p "$package_target" "$docs_artifact" "$package_lists" "$extracted_packages"

CARGO_BUILD_JOBS="$release_build_jobs" just release-audit

for package in "${publishable_packages[@]}"; do
  package_patch_args=()

  for dependency in "${publishable_packages[@]}"; do
    if [[ "$dependency" == "$package" ]]; then
      break
    fi

    extracted_root="${extracted_package_roots[$dependency]:-}"

    if [[ -n "$extracted_root" ]]; then
      package_patch_args+=(--config "patch.crates-io.${dependency}.path=\"${extracted_root}\"")
    fi
  done

  cargo package --allow-dirty --list -p "$package" "${package_patch_args[@]}" > "${package_lists}/${package}.txt"
  cargo package --allow-dirty --no-verify -p "$package" --target-dir "$package_target" "${package_patch_args[@]}"

  crate_archive="${package_target}/package/${package}-${version}.crate"
  tar -xzf "$crate_archive" -C "$extracted_packages"
  extracted_package_roots["$package"]="${extracted_packages}/${package}-${version}"

  package_check_args=(
    --offline
    --manifest-path "${extracted_package_roots[$package]}/Cargo.toml"
    --target-dir "$package_target"
  )

  package_check_args+=("${package_patch_args[@]}")

  cargo check "${package_check_args[@]}"
done

just test-package
mdbook build docs/book

cp -r target/book/. "$docs_artifact/"
cp "$package_target"/package/*.crate "$artifact_root/"

printf 'release verification artifacts: %s\n' "$artifact_root"
