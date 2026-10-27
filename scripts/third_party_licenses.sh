#!/usr/bin/env bash
# Regenerates assets/licenses/THIRD_PARTY_LICENSES.md: cargo-about's license texts, then the
# NOTICE files of the crates listed there, which Apache-2.0 asks redistributions to carry and
# cargo-about does not collect.
set -euo pipefail
cd "$(dirname "$0")/.."
out=assets/licenses/THIRD_PARTY_LICENSES.md
cargo about generate --fail -o "$out" about.hbs

# "name version" of every crate the notices list, then each one's source directory.
listed=$(grep -oE '^- [A-Za-z0-9_-]+ [0-9][^ ]*' "$out" | sed 's/^- //' | sort -u)
dirs=$(cargo metadata --format-version 1 --locked |
  jq -r '.packages[] | "\(.name) \(.version) \(.manifest_path)"' |
  awk 'NR == FNR { want[$1 " " $2]; next } ($1 " " $2) in want { sub(/\/Cargo.toml$/, "", $3); print $1, $2, $3 }' \
    <(echo "$listed") - | sort)

header=0
while read -r name version dir; do
  for file in NOTICE NOTICE.txt NOTICE.md; do
    [ -f "$dir/$file" ] || continue
    if [ "$header" = 0 ]; then
      printf '## Notices\n\nNOTICE files of the components above, reproduced as their licenses ask.\n\n' >>"$out"
      header=1
    fi
    printf '### %s %s\n\n```text\n%s\n```\n\n' "$name" "$version" "$(cat "$dir/$file")" >>"$out"
  done
done <<<"$dirs"
