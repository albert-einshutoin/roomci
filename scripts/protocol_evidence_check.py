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
B_TIER_TEXT = (ROOT / "docs" / "B_TIER_PROTOCOL_PROFILES.md").read_text()
SOURCE_TEXT = "\n".join(
    path.read_text(errors="ignore")
    for directory in ["crates", "examples", "adapter-contracts", "scripts", ".github"]
    for path in (ROOT / directory).rglob("*")
    if path.is_file()
)


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


def require_test_name(test_name):
    if test_name not in SOURCE_TEXT:
        fail(f"referenced evidence test is missing from source: {test_name}")


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
        for example in claim.get("examples", []):
            require_file(example)
        for test_name in claim.get("test_names", []):
            require_test_name(test_name)

        commands = claim.get("evidence_commands", [])
        if statuses & {"conformance_subset", "external_client_tested"} and not commands:
            fail(f"{name} is externally verified but has no evidence command")
        if statuses & {"scenario_model"} and not (
            claim.get("examples") and claim.get("test_names")
        ):
            fail(f"{name} is a scenario model but lacks example/test evidence")
        if statuses & {"adapter_sample_tested"} and not (
            {"make adapter-samples-smoke", "make python-sdk-smoke"} & set(commands)
        ):
            fail(f"{name} is adapter-sample tested but lacks an adapter smoke command")
        if statuses & {"unsupported", "future_profile"} and not claim.get("non_goal_docs"):
            fail(f"{name} is unsupported/future but has no non-goal docs")
        if statuses & {"contract_profile"}:
            if "conformance_subset" in statuses:
                fail(f"{name} cannot be both contract_profile and conformance_subset")
            if "make protocol-profile-smoke" not in commands:
                fail(f"{name} is a contract profile but lacks make protocol-profile-smoke")
            if not (claim.get("examples") and claim.get("non_goal_docs")):
                fail(f"{name} is a contract profile but lacks examples/non-goal docs")
            for token in ["not certification", "Non-goals", "contract_profile"]:
                if token not in B_TIER_TEXT:
                    fail(f"B Tier profile docs missing required wording: {token}")
        for command in commands:
            require_command(command)

    print(f"protocol evidence ok: {len(claims)} claims checked")


if __name__ == "__main__":
    main()
