#!/bin/sh
set -eu

root_dir=$(cd -- "$(dirname "$0")/.." && pwd)
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT
mkdir -p "$temp_dir/bin"
mkdir -p "$temp_dir/workspace/examples"

printf '%s\n' '#!/bin/sh' 'printf "%s\n" "$@"' > "$temp_dir/bin/roomci"
chmod 0755 "$temp_dir/bin/roomci"

GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "examples/one.yaml examples/two.yaml" \
  "roomci-reports" \
  "--quiet --run-id ci-42" > "$temp_dir/actual"

printf '%s\n' \
  run \
  --quiet \
  --run-id \
  ci-42 \
  --report-dir \
  roomci-reports \
  -- \
  examples/one.yaml \
  examples/two.yaml > "$temp_dir/expected"
diff -u "$temp_dir/expected" "$temp_dir/actual"

if GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "examples/one.yaml" \
  "../outside" \
  "" >/dev/null 2>&1; then
  echo "error: entrypoint accepted a report directory outside GITHUB_WORKSPACE" >&2
  exit 1
fi

if GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "examples/one.yaml" \
  "roomci-reports" \
  "--report-json arbitrary.json" >/dev/null 2>&1; then
  echo "error: entrypoint accepted an output-path flag through extra-args" >&2
  exit 1
fi

if GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "--report-json stolen.json examples/one.yaml" \
  "roomci-reports" \
  "" >/dev/null 2>&1; then
  echo "error: entrypoint accepted a CLI option through scenarios" >&2
  exit 1
fi

ln -s "$temp_dir/outside" "$temp_dir/workspace/symlink-reports"
if GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "examples/one.yaml" \
  "symlink-reports" \
  "" >/dev/null 2>&1; then
  echo "error: entrypoint accepted a symlinked report directory" >&2
  exit 1
fi

ln -s "$temp_dir/outside.yaml" "$temp_dir/workspace/examples/symlink.yaml"
if GITHUB_WORKSPACE="$temp_dir/workspace" PATH="$temp_dir/bin:$PATH" \
  sh "$root_dir/scripts/github-action-entrypoint.sh" \
  "examples/symlink.yaml" \
  "roomci-reports" \
  "" >/dev/null 2>&1; then
  echo "error: entrypoint accepted a symlinked scenario path" >&2
  exit 1
fi

echo "GitHub Action entrypoint contract is valid."
