#!/usr/bin/env bash
# Fails when the web build's brotli-compressed wasm exceeds the size budget.
#
# Usage: scripts/check_wasm_size.sh [dist dir] [budget file]
# Defaults: dist/ and .github/wasm-size-budget (one line: a byte count).
# Raise the budget only in a dedicated commit that says why the size grew.
set -euo pipefail

dist="${1:-dist}"
budget_file="${2:-.github/wasm-size-budget}"

shopt -s nullglob
wasm_files=("$dist"/*_bg.wasm)
if [ "${#wasm_files[@]}" -ne 1 ]; then
    echo "error: expected one *_bg.wasm in $dist, found ${#wasm_files[@]}" >&2
    exit 2
fi
wasm="${wasm_files[0]}"

budget="$(tr -d '[:space:]' < "$budget_file")"
if ! [[ "$budget" =~ ^[0-9]+$ ]]; then
    echo "error: $budget_file must hold a byte count, got '$budget'" >&2
    exit 2
fi

raw="$(wc -c < "$wasm" | tr -d ' ')"
gz="$(gzip -9c "$wasm" | wc -c | tr -d ' ')"
br="$(brotli -c -q 11 "$wasm" | wc -c | tr -d ' ')"

mib() { awk -v b="$1" 'BEGIN { printf "%.2f MiB", b / 1048576 }'; }
pct="$(awk -v s="$br" -v b="$budget" 'BEGIN { printf "%.1f", 100 * s / b }')"

report="$(cat <<EOF
| wasm | bytes | size |
|---|---:|---:|
| raw | $raw | $(mib "$raw") |
| gzip -9 | $gz | $(mib "$gz") |
| brotli -q 11 | $br | $(mib "$br") |
| **budget (brotli)** | $budget | $(mib "$budget") |

brotli size is ${pct}% of the budget.
EOF
)"
echo "$report"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    { echo "### Web build size"; echo; echo "$report"; } >> "$GITHUB_STEP_SUMMARY"
fi

if [ "$br" -gt "$budget" ]; then
    echo "error: compressed wasm ($(mib "$br")) exceeds the budget ($(mib "$budget"))." >&2
    echo "Shrink the build, or raise $budget_file in a commit that explains the growth." >&2
    exit 1
fi
