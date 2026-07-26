#!/bin/sh
set -eu

# Docker action arguments are passed as three strings. Intentional shell word
# splitting supports documented space-separated inputs. Globbing is disabled,
# and every output path and optional flag is validated before execution because
# this GitHub-required root container can write to the checked-out workspace.
set -f
scenarios=${1:?missing required scenarios input}
report_dir=${2:?missing report-dir input}
extra_args=${3-}
workspace=${GITHUB_WORKSPACE:-/github/workspace}

case "$report_dir" in
  "" | -* | /* | .. | ../* | */.. | */../*)
    echo "error: report-dir must be a relative path within GITHUB_WORKSPACE" >&2
    exit 2
    ;;
esac

workspace_root=$(CDPATH='' cd -- "$workspace" && pwd -P)
cd "$workspace_root"

# Reject every existing symlink component before resolving the output
# directory. This prevents the root action container from following a
# repository-controlled link beyond the mounted workspace.
validate_no_symlink_component() {
  candidate=$1
  current=.
  old_ifs=$IFS
  IFS=/
  # shellcheck disable=SC2086 # path components are intentionally split on /
  set -- $candidate
  IFS=$old_ifs
  for component in "$@"; do
    current=$current/$component
    if [ -L "$current" ]; then
      echo "error: symlink path components are unsupported: $candidate" >&2
      exit 2
    fi
  done
}

validate_no_symlink_component "$report_dir"
mkdir -p -- "$report_dir"
report_root=$(CDPATH='' cd -- "$report_dir" && pwd -P)
case "$report_root" in
  "$workspace_root" | "$workspace_root"/*) ;;
  *)
    echo "error: report-dir resolves outside GITHUB_WORKSPACE" >&2
    exit 2
    ;;
esac

set -- roomci run
expect_run_id=false
# shellcheck disable=SC2086 # documented space-separated safe flags
for token in $extra_args; do
  if [ "$expect_run_id" = true ]; then
    case "$token" in
      "" | -*)
        echo "error: --run-id requires a non-option value" >&2
        exit 2
        ;;
    esac
    set -- "$@" --run-id "$token"
    expect_run_id=false
    continue
  fi

  case "$token" in
    --verbose | --quiet | --dry-run)
      set -- "$@" "$token"
      ;;
    --run-id)
      expect_run_id=true
      ;;
    *)
      echo "error: unsupported extra-args token: $token" >&2
      exit 2
      ;;
  esac
done

if [ "$expect_run_id" = true ]; then
  echo "error: --run-id requires a value" >&2
  exit 2
fi

set -- "$@" --report-dir "$report_dir" --
# `--` is the primary option boundary; rejecting leading dashes also produces a
# clear error instead of silently treating a malformed scenario as a CLI flag.
# shellcheck disable=SC2086 # documented space-separated scenario paths
for scenario in $scenarios; do
  case "$scenario" in
    "" | -* | /* | .. | ../* | */.. | */../*)
      echo "error: scenario paths must stay within GITHUB_WORKSPACE: $scenario" >&2
      exit 2
      ;;
  esac
  validate_no_symlink_component "$scenario"
  set -- "$@" "$scenario"
done

exec "$@"
