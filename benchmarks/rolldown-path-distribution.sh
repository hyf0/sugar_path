#!/usr/bin/env bash
set -euo pipefail

export LC_ALL=C

repo=${1:?usage: bash benchmarks/rolldown-path-distribution.sh /path/to/rolldown [commit]}
commit=${2:-b9823050bc658ef65105148ea0504d4fbda7fa4c}
paths=$(mktemp)
lengths=$(mktemp)
components=$(mktemp)
trap 'rm -f "$paths" "$lengths" "$components"' EXIT

git -C "$repo" ls-tree -rz --name-only "$commit" >"$paths"

summarize() {
  sort -n | awk '
    { values[NR] = $1; sum += $1 }
    END {
      printf "n=%d min=%d p50=%d p90=%d p95=%d p99=%d max=%d mean=%.1f\n",
        NR,
        values[1],
        values[int(NR * 0.50)],
        values[int(NR * 0.90)],
        values[int(NR * 0.95)],
        values[int(NR * 0.99)],
        values[NR],
        sum / NR
    }
  '
}

total=0
lexically_clean=0
windows_raw_forward_clean=0
while IFS= read -r -d '' path; do
  total=$((total + 1))
  printf '%d\n' "${#path}" >>"$lengths"
  separators=${path//[^\/]/}
  printf '%d\n' "$(( ${#separators} + 1 ))" >>"$components"

  if [[ $path != /* && $path != */ && $path != *//* && "/$path/" != */./* && "/$path/" != */../* ]]; then
    lexically_clean=$((lexically_clean + 1))
  fi
  if [[ $path != */* ]]; then
    windows_raw_forward_clean=$((windows_raw_forward_clean + 1))
  fi
done <"$paths"

printf 'repository-relative bytes: '
sort -n "$lengths" | summarize

printf 'components: '
sort -n "$components" | summarize

printf 'repository-relative lexical clean paths: %d/%d\n' "$lexically_clean" "$total"
printf 'Windows borrowed hits if raw Git `/` separators are passed unchanged: %d/%d\n' \
  "$windows_raw_forward_clean" "$total"
