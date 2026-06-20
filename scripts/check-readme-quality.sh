#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TEST_LIST_FILE="$(mktemp)"
TARPAULIN_OUTPUT_DIR="$(mktemp -d)"
TARPAULIN_OUTPUT="$TARPAULIN_OUTPUT_DIR/tarpaulin-report.json"
trap 'rm -rf "$TEST_LIST_FILE" "$TARPAULIN_OUTPUT_DIR"' EXIT

cargo test --workspace --all-targets -- --list > "$TEST_LIST_FILE"
measured_tests=$(awk -F': ' '$2 == "test" { count++ } END { print count }' "$TEST_LIST_FILE")

cargo tarpaulin --workspace --engine llvm --fail-under 80 --out Json --output-dir "$TARPAULIN_OUTPUT_DIR" > "$TARPAULIN_OUTPUT"
measured_coverage=$(rg -oP "[0-9]+\.[0-9]+% coverage" "$TARPAULIN_OUTPUT" | tail -n 1 | sed -E 's/% coverage//')

if [[ -z "$measured_tests" || -z "$measured_coverage" ]]; then
  echo "Error: failed to capture live quality measurements." >&2
  exit 1
fi

extract_readme_values() {
  local file="$1"
  local lang="$2"

  if [[ "$lang" == "en" ]]; then
    tests=$(perl -ne 'if (/Current measurements:\s*\*\*([0-9]+) tests\*\*/) { print "$1\n"; exit }' "$file")
    coverage=$(perl -ne 'if (/Current measurements:.*\*\*([0-9]+\.[0-9]+)%\*\*/) { print "$1\n"; exit }' "$file")
    badge=$(perl -ne 'if (/coverage-([0-9]+\.[0-9]+)%25\)?:?/) { print "$1\n"; exit }' "$file")
  else
    tests=$(perl -ne 'if (/現在の測定値：\*\*([0-9]+) テスト\*\*/) { print "$1\n"; exit }' "$file")
    coverage=$(perl -ne 'if (/現在の測定値：.*\*\*([0-9]+\.[0-9]+)%\*\*/) { print "$1\n"; exit }' "$file")
    badge=$(perl -ne 'if (/coverage-([0-9]+\.[0-9]+)%25\)?:?/) { print "$1\n"; exit }' "$file")
  fi

  if [[ -z "$tests" || -z "$coverage" || -z "$badge" ]]; then
    echo "Error: unable to parse README metrics from $file" >&2
    exit 1
  fi

  echo "$tests $coverage $badge"
}

readme_en_metrics=$(extract_readme_values README.md en)
readme_ja_metrics=$(extract_readme_values README.ja.md ja)

readme_en_tests=$(echo "$readme_en_metrics" | awk '{print $1}')
readme_en_coverage=$(echo "$readme_en_metrics" | awk '{print $2}')
readme_en_badge=$(echo "$readme_en_metrics" | awk '{print $3}')

readme_ja_tests=$(echo "$readme_ja_metrics" | awk '{print $1}')
readme_ja_coverage=$(echo "$readme_ja_metrics" | awk '{print $2}')
readme_ja_badge=$(echo "$readme_ja_metrics" | awk '{print $3}')

if [[ "$measured_tests" != "$readme_en_tests" || "$measured_coverage" != "$readme_en_coverage" || "$measured_coverage" != "$readme_en_badge" ]]; then
  echo "Error: README.md quality claims are stale."
  echo "  tests: measured=$measured_tests expected=$readme_en_tests"
  echo "  coverage: measured=$measured_coverage expected=$readme_en_coverage"
  echo "  badge: measured=$measured_coverage expected=$readme_en_badge"
  exit 1
fi

if [[ "$measured_tests" != "$readme_ja_tests" || "$measured_coverage" != "$readme_ja_coverage" || "$measured_coverage" != "$readme_ja_badge" ]]; then
  echo "Error: README.ja.md quality claims are stale."
  echo "  tests: measured=$measured_tests expected=$readme_ja_tests"
  echo "  coverage: measured=$measured_coverage expected=$readme_ja_coverage"
  echo "  badge: measured=$measured_coverage expected=$readme_ja_badge"
  exit 1
fi

echo "README quality metrics are in sync: ${measured_tests} tests, ${measured_coverage}% line coverage."
