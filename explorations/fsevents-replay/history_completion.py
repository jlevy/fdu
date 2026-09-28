"""Diagnose completion against a preserved synthetic fixture with a new helper."""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import platform
import subprocess
import time
import unittest
from pathlib import Path
from typing import Any, cast

# This standalone directory is not an installed Python package.
run = importlib.import_module("run")

PARENT_GRACE_SECONDS = 10


def parse_records(stdout: str) -> tuple[list[dict[str, Any]], list[str]]:
    """A kill may split the last JSON line; preserve complete records and name the gap."""
    records: list[dict[str, Any]] = []
    errors: list[str] = []
    for number, line in enumerate(stdout.splitlines(), start=1):
        try:
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError("record is not an object")
            record = cast(dict[str, Any], value)
            if record.get("type") not in (
                "identity",
                "event",
                "summary",
            ):
                raise ValueError("unrecognized record")
            records.append(record)
        except (json.JSONDecodeError, ValueError) as error:
            errors.append(f"line {number}: {type(error).__name__}")
    return records, errors


def diagnose(fixture: Path, output: Path, seconds: int, mode: str, flags: int) -> None:
    """Keep capture provenance separate from the explicitly different replay instrument."""
    replay_sources = run.source_hashes()
    diagnostic_hash = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    state = json.loads((fixture / "state.json").read_text())
    if state["schema"] != run.SCHEMA:
        raise ValueError("unsupported fixture schema")
    old_binary = fixture / "probe"
    if hashlib.sha256(old_binary.read_bytes()).hexdigest() != state["provenance"]["binary_sha256"]:
        raise ValueError("recorded capture helper changed")
    if output.exists():
        raise ValueError("use a new output directory; preserve previous evidence")
    output.mkdir(mode=0o700)
    source = Path(__file__).with_name("probe.c")
    binary = output / "probe"
    subprocess.run(
        [
            "xcrun",
            "clang",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-fblocks",
            "-O2",
            str(source),
            "-framework",
            "CoreServices",
            "-o",
            str(binary),
        ],
        check=True,
        timeout=60,
    )
    if (
        run.source_hashes() != replay_sources
        or hashlib.sha256(Path(__file__).read_bytes()).hexdigest() != diagnostic_hash
    ):
        raise ValueError("instrument source changed during compilation; retry in a new directory")
    compiler = subprocess.check_output(["xcrun", "clang", "--version"], text=True).splitlines()[0]
    sdk = subprocess.check_output(["xcrun", "--show-sdk-version"], text=True).strip()
    root = fixture / "fixture"
    before = run.scan(root)
    started = time.time()
    args = [
        str(binary),
        "replay",
        str(root),
        str(state["captured"]["cursor"]),
        str(flags),
        str(seconds),
        mode,
    ]
    killed = False
    try:
        result = subprocess.run(
            args,
            capture_output=True,
            text=True,
            timeout=seconds + PARENT_GRACE_SECONDS,
            check=False,
        )
        stdout, stderr, returncode = result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as error:
        stdout = (error.stdout or b"").decode(errors="replace")
        stderr = (error.stderr or b"").decode(errors="replace")
        returncode = None
        killed = True
    wall = time.time() - started
    records, parse_errors = parse_records(stdout)
    if parse_errors:
        # Includes native identity; keep this private file out of published observations.
        run.save(output / "partial-output-private.json", {"stdout": stdout, "stderr": stderr})
    identities = [r for r in records if r["type"] == "identity"]
    events = [r for r in records if r["type"] == "event"]
    summaries = [r for r in records if r["type"] == "summary"]
    same_identity = bool(identities) and all(
        identities[0][key] == state["captured"][key] for key in ("device", "journal_uuid")
    )
    dirty, refused = run.scopes(events, bool(flags & run.FILE_EVENTS))
    after = run.scan(root)
    candidate = run.reconcile(root, state["baseline"], dirty)
    mismatches = run.differences(candidate, after)
    changed_during = run.differences(before, after)
    completed = bool(summaries) and summaries[-1]["history_done"] and returncode == 0
    source_unchanged = (
        run.source_hashes() == replay_sources
        and hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == diagnostic_hash
    )
    passed = (
        completed
        and same_identity
        and not refused
        and not mismatches
        and not changed_during
        and not parse_errors
        and source_unchanged
    )
    report = {
        "schema": "fdu-history-completion-diagnostic-v1",
        "platform": platform.mac_ver()[0],
        "architecture": platform.machine(),
        "capture_provenance": state["provenance"],
        "replay_provenance": {
            "source_sha256": replay_sources,
            "diagnostic_sha256": diagnostic_hash,
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "compiler": compiler,
            "sdk_version": sdk,
        },
        "source_unchanged": source_unchanged,
        "parse_errors": parse_errors,
        "scenario": state.get("scenario", "quiet"),
        "started_at": started,
        "cursor_age_seconds": started - state["captured_at"],
        "mutation_age_seconds": started - state["mutation_at"] if state["mutation_at"] else None,
        "deadline_seconds": seconds,
        "initial_flush": mode,
        "create_flags": flags,
        "cursor": state["captured"]["cursor"],
        "wall_seconds": wall,
        "parent_killed": killed,
        "returncode": returncode,
        "stderr": stderr,
        "identity_matches": same_identity,
        "events": events,
        "summary": summaries[-1] if summaries else None,
        "dirty_scopes": dirty,
        "refused": refused,
        "mismatch_paths": mismatches,
        "changed_during_trial": changed_during,
        "negative_control_mismatch_paths": run.differences(state["baseline"], after),
        "accepted": passed,
        "limitations": [
            "Synthetic fixture only",
            "No reboot or loss injection",
            "Matching oracle is not universal completeness proof",
        ],
    }
    run.save(output / "result.json", report)
    print(
        json.dumps(
            {
                key: report[key]
                for key in (
                    "cursor_age_seconds",
                    "deadline_seconds",
                    "initial_flush",
                    "create_flags",
                    "wall_seconds",
                    "parent_killed",
                    "summary",
                    "mismatch_paths",
                    "accepted",
                )
            },
            indent=2,
        )
    )
    if not passed:
        raise SystemExit(1)


class DiagnosticTests(unittest.TestCase):
    def test_partial_timeout_output_preserves_completed_records(self) -> None:
        records, errors = parse_records('{"type":"event","id":1}\n{"type":"summary"')
        self.assertEqual(records, [{"type": "event", "id": 1}])
        self.assertEqual(errors, ["line 2: JSONDecodeError"])
        records, errors = parse_records('null\n[]\n{"type":"unknown"}\n{"type":"summary"}\n')
        self.assertEqual(records, [{"type": "summary"}])
        self.assertEqual(len(errors), 3)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--seconds", type=int, choices=range(1, 121), default=10)
    parser.add_argument("--mode", choices=("wait", "async", "sync"), default="wait")
    parser.add_argument("--flags", type=int, choices=(0, 16, 128, 144), default=144)
    args = parser.parse_args()
    diagnose(args.fixture, args.output, args.seconds, args.mode, args.flags)


if __name__ == "__main__":
    main()
