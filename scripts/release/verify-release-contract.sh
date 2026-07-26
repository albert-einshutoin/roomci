#!/bin/sh
set -eu

workspace_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
if [ -z "$workspace_version" ]; then
  echo 'error: workspace version is missing from Cargo.toml' >&2
  exit 1
fi

if [ "$#" -eq 1 ]; then
  tag=$1
  if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "error: release tag must be vMAJOR.MINOR.PATCH, got: $tag" >&2
    exit 1
  fi
  if [ "${tag#v}" != "$workspace_version" ]; then
    echo "error: tag $tag does not match Cargo workspace version $workspace_version" >&2
    exit 1
  fi
elif [ "$#" -ne 0 ]; then
  echo 'usage: verify-release-contract.sh [vMAJOR.MINOR.PATCH]' >&2
  exit 2
fi

# Cargo metadata is the authoritative resolver check before a runner packages
# a binary. This keeps tag/version validation independent of release tooling.
cargo metadata --locked --format-version 1 --no-deps >/dev/null
echo "Release contract valid for workspace version $workspace_version."
