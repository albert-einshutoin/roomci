#!/bin/sh
set -eu

target=${1:?usage: package.sh <target-triple> <binary-path> <output-directory>}
binary=${2:?usage: package.sh <target-triple> <binary-path> <output-directory>}
output_dir=${3:?usage: package.sh <target-triple> <binary-path> <output-directory>}
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)

case "$target" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-apple-darwin|aarch64-apple-darwin) ;;
  *)
    echo "error: unsupported release target: $target" >&2
    exit 2
    ;;
esac

if [ ! -x "$binary" ]; then
  echo "error: release binary is missing or not executable: $binary" >&2
  exit 1
fi

package_name="roomci-v${version}-${target}"
archive="$output_dir/${package_name}.tar.gz"
stage_dir=$(mktemp -d)
trap 'rm -rf "$stage_dir"' EXIT
mkdir -p "$output_dir" "$stage_dir/$package_name/examples"
install -m 0755 "$binary" "$stage_dir/$package_name/roomci"
install -m 0644 README.md LICENSE "$stage_dir/$package_name/"
# Keep the validation example beside the binary so the documented first-run
# check works from an extracted release archive without a source checkout.
install -m 0644 examples/local_first_cloud_outage.yaml "$stage_dir/$package_name/examples/"
tar -C "$stage_dir" -czf "$archive" "$package_name"
printf '%s\n' "$archive"
