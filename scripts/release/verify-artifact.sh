#!/bin/sh
set -eu

artifact_dir=${1:?usage: verify-artifact.sh <artifact-dir> <target-triple>}
target=${2:?usage: verify-artifact.sh <artifact-dir> <target-triple>}
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)

case "$target" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin) ;;
  *)
    echo "error: unsupported release target: $target" >&2
    exit 2
    ;;
esac

archive="$artifact_dir/roomci-v${version}-${target}.tar.gz"

test -f "$archive"
contents=$(tar -tzf "$archive")
printf '%s\n' "$contents" | grep -Fx "roomci-v${version}-${target}/roomci" >/dev/null
printf '%s\n' "$contents" | grep -Fx "roomci-v${version}-${target}/README.md" >/dev/null
printf '%s\n' "$contents" | grep -Fx "roomci-v${version}-${target}/LICENSE" >/dev/null
printf '%s\n' "$contents" | grep -Fx "roomci-v${version}-${target}/examples/local_first_cloud_outage.yaml" >/dev/null

stage_dir=$(mktemp -d)
trap 'rm -rf "$stage_dir"' EXIT
tar -C "$stage_dir" -xzf "$archive"
binary="$stage_dir/roomci-v${version}-${target}/roomci"
"$binary" --version | grep -F "roomci $version" >/dev/null
"$binary" validate "$stage_dir/roomci-v${version}-${target}/examples/local_first_cloud_outage.yaml" >/dev/null
echo "Release artifact valid: $archive"
