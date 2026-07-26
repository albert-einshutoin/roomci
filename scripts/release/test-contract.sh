#!/bin/sh
set -eu

sh scripts/release/check-contract.sh
temp_dir=$(mktemp -d)
trap 'rm -rf "$temp_dir"' EXIT
sed "s/startsWith(github.ref_name, 'v')/false/g" .github/workflows/release.yml > "$temp_dir/release.yml"
if RELEASE_WORKFLOW="$temp_dir/release.yml" ruby scripts/release/check-contract.rb >/dev/null 2>&1; then
  echo 'error: release contract accepted a workflow_dispatch-capable publish job without a tag guard' >&2
  exit 1
fi

sed "s/github.event_name == 'push'/true/g" .github/workflows/release.yml > "$temp_dir/release.yml"
if RELEASE_WORKFLOW="$temp_dir/release.yml" ruby scripts/release/check-contract.rb >/dev/null 2>&1; then
  echo 'error: release contract accepted publication from a manual tag dispatch' >&2
  exit 1
fi

echo 'Release contract negative test is valid.'
