#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
exec bash "$script_dir/check-doc-hygiene.sh"
