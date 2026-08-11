#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

artifact_root="target/vvm-release-audit"
rm -rf "$artifact_root"
mkdir -p "$artifact_root"

versions_file="$artifact_root/tool-versions.txt"

record_tool_version() {
    local label="$1"
    shift

    if ! command -v "$1" >/dev/null 2>&1; then
        return
    fi

    {
        printf '## %s\n' "$label"
        "$@"
        printf '\n'
    } >> "$versions_file" 2>&1
}

run_and_capture() {
    local name="$1"
    shift
    local log_file="$artifact_root/${name}.log"

    printf '==> %s\n' "$name"
    "$@" 2>&1 | tee "$log_file"
}

: > "$versions_file"

record_tool_version "rustc" rustc --version
record_tool_version "cargo" cargo --version
record_tool_version "cargo fmt (nightly)" cargo +nightly fmt --version
record_tool_version "cargo nextest" cargo nextest --version
record_tool_version "cargo audit" cargo audit --version
record_tool_version "cargo deny" cargo deny --version
record_tool_version "cargo udeps" cargo udeps -V
record_tool_version "cargo public-api" cargo public-api --version
record_tool_version "mdbook" mdbook -V
record_tool_version "verilator" verilator --version
record_tool_version "g++" g++ --version
record_tool_version "clang++" clang++ --version

run_and_capture ci just ci
run_and_capture deny just deny
run_and_capture api-diff just api-diff
run_and_capture repro-check just repro-check

printf 'release audit artifacts: %s\n' "$artifact_root"
