#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

book_src_dir="$root/docs/book/src"
public_dir="$root/public"
summary_path="$book_src_dir/SUMMARY.md"
site_base="$(perl -nle 'print $1 and exit if /^site-url = "([^"]+)"$/' docs/book/book.toml)"

failures=0
declare -A indexed_sources=()

record_failure() {
    local title="$1"
    local source="$2"
    local target="${3:-}"
    local detail="${4:-}"

    printf '%s\n' "$title" >&2
    printf '  %s\n' "$source" >&2

    if [[ -n "$target" ]]; then
        printf '  %s\n' "$target" >&2
    fi

    if [[ -n "$detail" ]]; then
        printf '  %s\n' "$detail" >&2
    fi

    failures=1
}

normalize_path() {
    realpath -m "$1"
}

source_to_public_html() {
    local source_path="$1"
    local relative_path="${source_path#"$book_src_dir/"}"

    printf '%s/%s\n' "$public_dir" "${relative_path%.md}.html"
}

html_has_anchor() {
    local html_path="$1"
    local anchor="$2"

    rg -qF "id=\"$anchor\"" "$html_path"
}

validate_html_anchor() {
    local title="$1"
    local source="$2"
    local target="$3"
    local html_path="$4"
    local anchor="$5"

    if ! html_has_anchor "$html_path" "$anchor"; then
        record_failure "$title" "$source" "$target" "missing id in ${html_path#"$root/"}"
    fi
}

check_summary_targets() {
    while IFS= read -r relative_path; do
        local source_path="$book_src_dir/$relative_path"

        indexed_sources["$source_path"]=1

        if [[ ! -f "$source_path" ]]; then
            record_failure "Missing SUMMARY target:" "$source_path" "referenced by docs/book/src/SUMMARY.md"
        fi
    done < <(perl -nle 'print $1 while /\(([^)#]+\.md)(?:#[^)]+)?\)/g' "$summary_path")
}

check_orphaned_sources() {
    while IFS= read -r source_path; do
        if [[ "$source_path" == "$summary_path" || "$source_path" == "$book_src_dir/404.md" ]]; then
            continue
        fi

        if [[ -n "${indexed_sources["$source_path"]:-}" ]]; then
            continue
        fi

        record_failure "Orphaned book chapter:" "${source_path#"$root/"}" "not referenced by docs/book/src/SUMMARY.md"
    done < <(rg --files docs/book/src -g '*.md' | while IFS= read -r path; do printf '%s/%s\n' "$root" "$path"; done)
}

check_source_links() {
    while IFS='|' read -r source_path line_number target; do
        local source_dir public_source_html public_source_dir base_target fragment resolved_source resolved_html relative_repo

        source_dir="$(dirname "$source_path")"
        public_source_html="$(source_to_public_html "$source_path")"
        public_source_dir="$(dirname "$public_source_html")"
        base_target="$target"
        fragment=''

        case "$target" in
            http://*|https://*|mailto:*)
                continue
                ;;
        esac

        if [[ "$target" == *'#'* ]]; then
            base_target="${target%%#*}"
            fragment="${target#*#}"

            if [[ "$base_target" == "$target" ]]; then
                base_target=''
            fi
        fi

        if [[ -z "$base_target" ]]; then
            validate_html_anchor "Broken book anchor:" "${source_path#"$root/"}:$line_number" "$target" "$public_source_html" "$fragment"
            continue
        fi

        if [[ "$base_target" == *.md ]]; then
            resolved_source="$(normalize_path "$source_dir/$base_target")"

            if [[ "$resolved_source" != "$book_src_dir/"* ]]; then
                relative_repo="$(realpath --relative-to="$root" "$resolved_source")"

                if [[ "$relative_repo" == docs/*.md || "$relative_repo" == docs/dev/*.md ]]; then
                    record_failure "Forbidden internal documentation link:" "${source_path#"$root/"}:$line_number" "$target"
                else
                    record_failure "Pages-invalid relative markdown link:" "${source_path#"$root/"}:$line_number" "$target" "use a published book or API page instead"
                fi

                continue
            fi

            if [[ ! -f "$resolved_source" ]]; then
                record_failure "Broken internal book link:" "${source_path#"$root/"}:$line_number" "$target"
                continue
            fi

            resolved_html="$(source_to_public_html "$resolved_source")"

            if [[ ! -f "$resolved_html" ]]; then
                record_failure "Missing rendered book target:" "${source_path#"$root/"}:$line_number" "$target" "expected ${resolved_html#"$root/"}"
                continue
            fi

            if [[ -n "$fragment" ]]; then
                validate_html_anchor "Broken book anchor:" "${source_path#"$root/"}:$line_number" "$target" "$resolved_html" "$fragment"
            fi

            continue
        fi

        if [[ "$base_target" == *.html ]]; then
            resolved_html="$(normalize_path "$public_source_dir/$base_target")"

            if [[ "$resolved_html" != "$public_dir/"* ]]; then
                record_failure "Pages-invalid relative HTML link:" "${source_path#"$root/"}:$line_number" "$target"
                continue
            fi

            if [[ ! -f "$resolved_html" ]]; then
                record_failure "Broken rendered HTML link:" "${source_path#"$root/"}:$line_number" "$target" "expected ${resolved_html#"$root/"}"
                continue
            fi

            if [[ -n "$fragment" ]]; then
                validate_html_anchor "Broken rendered HTML anchor:" "${source_path#"$root/"}:$line_number" "$target" "$resolved_html" "$fragment"
            fi
        fi
    done < <(
        while IFS= read -r source_path; do
            perl -ne 'while (/\[[^][]+\]\(([^)]+)\)/g) { print "$ARGV|$.|$1\n"; }' "$source_path"
        done < <(printf '%s\n' "${!indexed_sources[@]}" | sort)
    )
}

check_redirects() {
    while IFS='|' read -r output_path destination; do
        local redirect_html base_destination fragment resolved_html

        redirect_html="$root/$output_path"

        if [[ ! -f "$redirect_html" ]]; then
            record_failure "Missing redirect output:" "$output_path" "$destination"
            continue
        fi

        base_destination="$destination"
        fragment=''

        if [[ "$destination" == *'#'* ]]; then
            base_destination="${destination%%#*}"
            fragment="${destination#*#}"
        fi

        resolved_html="$(normalize_path "$(dirname "$redirect_html")/$base_destination")"

        if [[ "$resolved_html" != "$public_dir/"* ]]; then
            record_failure "Pages-invalid redirect destination:" "$output_path" "$destination"
            continue
        fi

        if [[ ! -f "$resolved_html" ]]; then
            record_failure "Broken redirect destination:" "$output_path" "$destination" "expected ${resolved_html#"$root/"}"
            continue
        fi

        if [[ -n "$fragment" ]]; then
            validate_html_anchor "Broken redirect anchor:" "$output_path" "$destination" "$resolved_html" "$fragment"
        fi
    done < <(perl -nle 'print "$1|$2" if /^write_redirect\s+(\S+)\s+(\S+)/' scripts/doc-suite.sh)
}

check_pages_base_path() {
    if ! rg -qF "<base href=\"$site_base\">" public/404.html; then
        record_failure "Broken Pages base path:" "public/404.html" "expected <base href=\"$site_base\">"
    fi
}

check_public_status_wording() {
    local -a public_docs=(
        "README.md"
        "crates/vvm/README.md"
        "crates/vvm-build/README.md"
        "crates/cargo-vvm/README.md"
        "docs/book/src/introduction.md"
        "docs/book/src/why-vvm.md"
        "docs/book/src/installation.md"
        "docs/book/src/quick-start.md"
        "docs/book/src/guide/compatibility-and-limitations.md"
        "docs/book/src/development/common-commands.md"
        "docs/book/src/development/architecture.md"
    )
    local status_pattern='early alpha|alpha software|alpha API stability'

    if rg -n -i --pcre2 "$status_pattern" "${public_docs[@]}" >/tmp/vvm-public-status-wording.log; then
        while IFS= read -r line; do
            record_failure "Obsolete public status wording:" "$line" "update current public docs to pre-1.0 release-cycle language"
        done < /tmp/vvm-public-status-wording.log
    fi
}

check_summary_targets
check_orphaned_sources
check_source_links
check_redirects
check_pages_base_path
check_public_status_wording

exit "$failures"
