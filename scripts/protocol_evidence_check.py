#!/usr/bin/env python3
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs" / "protocol-evidence.json"
REGISTRY_TEXT = (ROOT / "docs" / "PROTOCOL_CONFORMANCE_REGISTRY.md").read_text()
SUPPORT_TEXT = (ROOT / "docs" / "PROTOCOL_SUPPORT_MATRIX.md").read_text()
RELEASE_TEXT = (ROOT / "docs" / "RELEASE_CHECKLIST.md").read_text()
MAKEFILE_TEXT = (ROOT / "Makefile").read_text()


def fail(message):
    print(f"protocol evidence check failed: {message}", file=sys.stderr)
    sys.exit(1)


def require_file(path):
    target = ROOT / path
    if not target.exists():
        fail(f"missing referenced evidence file: {path}")


def require_command(command):
    haystack = "\n".join([REGISTRY_TEXT, SUPPORT_TEXT, RELEASE_TEXT])
    if command not in haystack:
        fail(f"evidence command is not documented in public protocol docs: {command}")
    if command.startswith("make "):
        target = command.split(maxsplit=1)[1]
        if f"{target}:" not in MAKEFILE_TEXT:
            fail(f"evidence make target is missing from Makefile: {target}")


def main():
    manifest = json.loads(MANIFEST.read_text())
    claims = manifest.get("claims", [])
    if not claims:
        fail("manifest has no claims")

    for claim in claims:
        name = claim.get("name", "<unnamed>")
        statuses = set(claim.get("status", []))
        for doc in claim.get("docs", []):
            require_file(doc)
        for doc in claim.get("non_goal_docs", []):
            require_file(doc)

        commands = claim.get("evidence_commands", [])
        if statuses & {"conformance_subset", "external_client_tested"} and not commands:
            fail(f"{name} is externally verified but has no evidence command")
        if statuses & {"unsupported", "future_profile"} and not claim.get("non_goal_docs"):
            fail(f"{name} is unsupported/future but has no non-goal docs")
        for command in commands:
            require_command(command)

    print(f"protocol evidence ok: {len(claims)} claims checked")


if __name__ == "__main__":
    main()
