#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

missing=0

while IFS= read -r link; do
    target="${link%%#*}"
    public_path="public/${target#../}"

    if [ ! -e "$public_path" ]; then
        printf 'Missing API Guide target: %s -> %s\n' "$link" "$public_path" >&2
        missing=1
    fi
done < <(rg -No --no-filename '\.\./api/[^)#]+' docs/book/src/api-guide/*.md | sort -u)

exit "$missing"
