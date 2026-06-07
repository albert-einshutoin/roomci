#!/usr/bin/env python3
"""Guard PoC targets against multi-scenario commands with single report files."""

from __future__ import annotations

import re
import shlex
import subprocess
import sys


TARGETS = [
    "protocol-profile-smoke",
    "poc-generic-mqtt",
    "poc-core-qa",
    "poc-hospitality",
    "poc-building-automation",
    "poc-bms-ops",
]

REPORT_FLAGS = {
    "--report-json",
    "--report-md",
    "--junit",
    "--timeline-json",
    "--timeline-ndjson",
    "--observability-json",
}

RUN_MARKER = ["cargo", "run", "-p", "roomci-cli", "--", "run"]


def make_dry_run() -> str:
    result = subprocess.run(
        ["make", "-n", *TARGETS],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout


def logical_recipe_lines(output: str) -> list[str]:
    lines: list[str] = []
    current = ""
    for raw in output.splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.endswith("\\"):
            current += line[:-1] + " "
            continue
        current += line
        lines.append(current)
        current = ""
    if current:
        lines.append(current)
    return lines


def run_segments(line: str) -> list[str]:
    return [
        segment
        for segment in re.split(r";\s*", line)
        if "cargo run -p roomci-cli -- run" in segment
    ]


def roomci_run_args(segment: str) -> list[str]:
    tokens = shlex.split(segment)
    for index in range(0, len(tokens) - len(RUN_MARKER) + 1):
        if tokens[index : index + len(RUN_MARKER)] == RUN_MARKER:
            return tokens[index + len(RUN_MARKER) :]
    return []


def violations(output: str) -> list[str]:
    found: list[str] = []
    for line in logical_recipe_lines(output):
        for segment in run_segments(line):
            args = roomci_run_args(segment)
            if not any(flag in args for flag in REPORT_FLAGS):
                continue
            literal_scenarios = [
                arg
                for arg in args
                if arg.startswith("examples/") and arg.endswith(".yaml")
            ]
            if len(literal_scenarios) > 1:
                found.append(segment)
    return found


def main() -> int:
    bad_segments = violations(make_dry_run())
    if bad_segments:
        print(
            "PoC report artifact contract failed: report flags must not be used "
            "with multiple literal scenarios in one roomci run command.",
            file=sys.stderr,
        )
        for segment in bad_segments:
            print(f"- {segment}", file=sys.stderr)
        return 1

    print("PoC report artifact contract valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
