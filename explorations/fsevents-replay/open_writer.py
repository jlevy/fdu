"""Bounded synthetic diagnostic: append while a writer stays open, then close it."""

from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import os
import platform
import subprocess
import tempfile
import time
import unittest
from dataclasses import asdict
from pathlib import Path
from typing import Any, cast

tree = importlib.import_module("real_tree")
fixture = importlib.import_module("fixture_workload")
Record = dict[str, Any]
FLAGS = 144
MAX_ENTRIES = 1024


def hashes() -> dict[str, str]:
    return {
        name: hashlib.sha256(Path(__file__).with_name(name).read_bytes()).hexdigest()
        for name in ("open_writer.py", "real_tree.py", "fixture_workload.py", "probe.c")
    }


def parse_records(stdout: str) -> tuple[list[Record], list[str]]:
    records: list[Record] = []
    malformed: list[str] = []
    for line in stdout.splitlines():
        try:
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError("record must be an object")
            records.append(cast(Record, value))
        except ValueError:
            malformed.append(line)
    return records, malformed


def replay(helper: Path, root: Path, cursor: int, seconds: int) -> Record:
    started = time.monotonic()
    killed = False
    try:
        result = subprocess.run(
            [str(helper), "replay", str(root), str(cursor), str(FLAGS), str(seconds), "wait"],
            capture_output=True,
            text=True,
            timeout=seconds + 10,
            check=False,
        )
        stdout, stderr, returncode = result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as error:
        stdout = (error.stdout or b"").decode(errors="replace")
        stderr = (error.stderr or b"").decode(errors="replace")
        returncode, killed = None, True
    records, malformed = parse_records(stdout)
    summaries = [record for record in records if record.get("type") == "summary"]
    summary = summaries[-1] if summaries else {}
    return {
        "records": records,
        "stderr": stderr,
        "malformed_lines": malformed,
        "parent_killed": killed,
        "returncode": returncode,
        "wall_seconds": time.monotonic() - started,
        "completed": bool(summary.get("history_done")) and returncode == 0 and not malformed,
        "summary": summary,
    }


def stage(
    helper: Path,
    root: Path,
    baseline: Record,
    expected: tuple[int, int],
    capture: Record,
    cursor: int,
    seconds: int,
    writer_open: bool,
    log_name: str,
) -> Record:
    before, before_stats = tree.full_scan(root, expected, MAX_ENTRIES)
    native = replay(helper, root, cursor, seconds)
    plan = tree.normalize_events(native["records"], FLAGS)
    candidate, candidate_stats = tree.reconcile(root, baseline, expected, plan, MAX_ENTRIES)
    after, after_stats = tree.full_scan(root, expected, MAX_ENTRIES)
    comparison = tree.compare_oracles(candidate, before, after)
    events: list[Record] = [
        record
        for record in native["records"]
        if record.get("type") == "event" and not record["flags"] & tree.HISTORY_DONE
    ]
    overlap = [event for event in events if event["id"] <= cursor]
    overlap_plan = tree.normalize_events(overlap, FLAGS)
    overlap_candidate, _ = tree.reconcile(root, baseline, expected, overlap_plan, MAX_ENTRIES)
    identities = [record for record in native["records"] if record.get("type") == "identity"]
    identity_matches = bool(identities) and all(
        identities[0][key] == capture[key] for key in ("device", "journal_uuid")
    )
    observation_error_count = sum(
        len(stats["errors"]) for stats in (before_stats, candidate_stats, after_stats)
    )
    valid = bool(
        native["completed"]
        and identity_matches
        and not observation_error_count
        and not plan.fallback_reasons
    )
    return {
        "writer_open": writer_open,
        "writer_pid": os.getpid(),
        "native": native,
        "identity_matches": identity_matches,
        "completed": native["completed"],
        "observation_error_count": observation_error_count,
        "degradation_reason_count": len(plan.fallback_reasons),
        "diagnostic_valid": valid,
        "scope_plan": asdict(plan),
        "candidate_stats": candidate_stats,
        "before_stats": before_stats,
        "after_stats": after_stats,
        "baseline_log": baseline[log_name],
        "before_log": before[log_name],
        "candidate_log": candidate[log_name],
        "after_log": after[log_name],
        "comparison": comparison,
        "negative_control": tree.compare_oracles(baseline, before, after),
        "overlap_event_count": len(overlap),
        "event_count": len(events),
        "fresh_log_nominations": sum(
            event["id"] > cursor and event["path_hex"] == os.fsencode(log_name).hex()
            for event in events
        ),
        "overlap_alone_observes_changed_log": overlap_candidate[log_name]
        == after[log_name]
        != baseline[log_name],
        "log_omission": valid and before[log_name] == after[log_name] != candidate[log_name],
        "verified_equal": valid and comparison["status"] == "equal",
    }


def owned_leaf_root(owned_root: Path, leaf: str) -> tuple[Path, str]:
    """Accept an explicit existing synthetic leaf, never a symlink or unmarked tree."""
    owned_root = owned_root.absolute()
    if len(owned_root.parts) < 5 or owned_root.parts[1] != "Volumes":
        raise ValueError("owned fixture must be on external scratch")
    if not os.path.ismount(Path(*owned_root.parts[:3])):
        raise ValueError("owned fixture volume not mounted")
    parts = tree.normalized_parts(leaf)
    if not parts:
        raise ValueError("explicit fixture leaf required")
    with (
        tree.directory_fd(owned_root) as descriptor,
        fixture.child_fd(descriptor, fixture.MARKER) as marker,
    ):
        if os.fstat(marker).st_size > 1024:
            raise ValueError("oversized fixture ownership marker")
        marker_state = json.loads(os.read(marker, 1025))
        if marker_state not in (
            {"schema": "fdu-growth-fixture-v1", "mutated": False},
            {"schema": "fdu-growth-fixture-v1", "mutated": True},
        ):
            raise ValueError("not an owned growth fixture")
    root = owned_root.joinpath(*parts[:-1])
    with tree.directory_fd(root) as descriptor, fixture.child_fd(descriptor, parts[-1]) as target:
        if os.fstat(target).st_nlink != 1:
            raise ValueError("existing diagnostic leaf must have exactly one hardlink")
    return root, parts[-1]


def diagnose(
    state: Path, helper: Path, seconds: int, owned_root: Path | None = None, leaf: str | None = None
) -> bool:
    state = state.absolute()
    if len(state.parts) < 5 or state.parts[1] != "Volumes":
        raise ValueError("new state directory must be on mounted external scratch")
    volume = Path(*state.parts[:3])
    if not os.path.ismount(volume):
        raise ValueError("external volume not mounted")
    if (owned_root is None) != (leaf is None):
        raise ValueError("existing fixture requires both --owned-fixture and --leaf")
    if owned_root is not None and leaf is not None:
        root, log_name = owned_leaf_root(owned_root, leaf)
        tree.validate_locations(root, state)
    else:
        root, log_name = state / "fixture", "log"
    sources = hashes()
    provenance = tree.helper_provenance(helper)
    with tree.directory_fd(state.parent) as parent:
        if os.fstat(parent).st_dev == os.stat(Path.home()).st_dev:
            raise ValueError("state must not resolve to internal home volume")
        os.mkdir(state.name, mode=0o700, dir_fd=parent)
    if owned_root is None:
        root.mkdir(mode=0o700)
    limits = tree.Limits(
        max_entries=MAX_ENTRIES, max_state_bytes=64 * 1024**2, reserve_free_bytes=2 * 1024**3
    )
    budget = state, limits
    tree.check_space(*budget)
    with tree.directory_fd(root) as root_fd:
        cursor: int | None = None
        if owned_root is None:
            fixture.write_new(root_fd, log_name, b"baseline\n")
        expected = tree.root_identity(root)
        if owned_root is None:
            initial = tree.native(helper, "capture", root)[0]
            drained = replay(helper, root, initial["device_fence"], seconds)
            tree.atomic_json(state / "creation-drain.json", drained, budget)
            if not drained["completed"]:
                raise ValueError("creation drain incomplete; fixture retained")
            # Synthetic creation-drain control, never a production boundary claim.
            cursor = int(drained["summary"]["latest_id"])
        with fixture.child_fd(root_fd, log_name) as writer:
            capture = tree.native(helper, "capture", root)[0]
            if cursor is None:
                cursor = int(capture["device_fence"])
            if not cursor:
                raise ValueError("invalid device fence")
            baseline, baseline_stats = tree.full_scan(root, expected, MAX_ENTRIES)
            if baseline_stats["errors"]:
                raise ValueError("baseline scan failed")
            writer_facts = tree.facts(os.fstat(writer))
            if (
                tree.identity(writer_facts) != tree.identity(baseline[log_name])
                or writer_facts.nlink != 1
            ):
                raise ValueError("writer leaf changed identity or acquired aliases")
            tree.atomic_json(
                state / "baseline.json",
                {
                    "fields": list(tree.Facts._fields),
                    "inventory": baseline,
                    "scan": baseline_stats,
                    "capture": capture,
                    "cursor": cursor,
                    "writer_pid": os.getpid(),
                    "writer_open": True,
                    "sources": sources,
                    "provenance": tree.public_provenance(provenance),
                },
                budget,
            )
            os.lseek(writer, 0, os.SEEK_END)
            payload = b"append held open\n" * 1024
            if os.write(writer, payload) != len(payload):
                raise OSError("short diagnostic append")
            os.fsync(writer)
            while_open = stage(
                helper, root, baseline, expected, capture, cursor, seconds, True, log_name
            )
            tree.atomic_json(state / "while-open.json", while_open, budget)
            os.fstat(writer)  # Confirm the writer descriptor survived the replay subprocess.
        after_close = stage(
            helper, root, baseline, expected, capture, cursor, seconds, False, log_name
        )
        tree.atomic_json(state / "after-close.json", after_close, budget)
    if hashes() != sources or tree.helper_provenance(helper) != provenance:
        raise ValueError("diagnostic sources or helper changed during trial; evidence unaccepted")
    summary = {
        "schema": "fdu-open-writer-diagnostic-v1",
        "sources": sources,
        "provenance": tree.public_provenance(provenance),
        "platform": platform.mac_ver()[0],
        "architecture": platform.machine(),
        "filesystem": capture["filesystem"],
        "cursor": cursor,
        "captured_device_fence": capture["device_fence"],
        "cursor_protocol": "current device-time fence before append"
        if owned_root is not None
        else "creation drain latest ID; both stages reuse unchanged cursor",
        "existing_owned_fixture": owned_root is not None,
        "create_flags": FLAGS,
        "deadline_seconds": seconds,
        "diagnostic_valid": while_open["diagnostic_valid"] and after_close["diagnostic_valid"],
        "open_omission_reproduced": while_open["log_omission"],
        "closed_stage_equal": after_close["verified_equal"],
        "overlap_masks_open_stage": while_open["overlap_alone_observes_changed_log"],
        "stages": {
            label: {
                key: result[key]
                for key in (
                    "writer_open",
                    "completed",
                    "identity_matches",
                    "observation_error_count",
                    "degradation_reason_count",
                    "diagnostic_valid",
                    "event_count",
                    "overlap_event_count",
                    "fresh_log_nominations",
                    "overlap_alone_observes_changed_log",
                    "log_omission",
                    "comparison",
                )
            }
            for label, result in (("open", while_open), ("closed", after_close))
        },
        "limitations": [
            "Synthetic external fixture, not application attribution",
            "Overlap coverage can mask a missing fresh notification",
            "HistoryDone is not a production safe cursor advancement proof",
        ],
    }
    tree.atomic_json(state / "summary.json", summary, budget)
    print(json.dumps(summary, indent=2))
    return bool(summary["diagnostic_valid"])


class DiagnosticTests(unittest.TestCase):
    def test_partial_and_nonobject_output_retained_as_malformed(self) -> None:
        records, malformed = parse_records('{"type":"identity"}\nnull\n[]\n{"partial":')
        self.assertEqual(records, [{"type": "identity"}])
        self.assertEqual(malformed, ["null", "[]", '{"partial":'])

    def test_owned_leaf_rejects_unmarked_and_symlink_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "branch").mkdir()
            target = root / "branch/file"
            target.write_bytes(b"owned")
            with self.assertRaises(OSError):
                owned_leaf_root(root, "branch/file")
            (root / fixture.MARKER).write_bytes(fixture.marker_payload(False))
            self.assertEqual(owned_leaf_root(root, "branch/file"), (root / "branch", "file"))
            with self.assertRaises(ValueError):
                owned_leaf_root(root, "../escape")
            target.rename(root / "original")
            target.symlink_to(root / "original")
            with self.assertRaises(OSError):
                owned_leaf_root(root, "branch/file")
            (root / "branch-alias").symlink_to(root / "branch", target_is_directory=True)
            with self.assertRaises(OSError):
                owned_leaf_root(root, "branch-alias/file")
            self.assertEqual((root / "original").read_bytes(), b"owned")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("state", type=Path, nargs="?")
    parser.add_argument("--helper", type=Path)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--seconds", type=int, choices=range(1, 61), default=10)
    parser.add_argument("--owned-fixture", type=Path)
    parser.add_argument("--leaf", help="Explicit relative file under the marked existing fixture")
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=[__file__])
        return
    if args.state is None or args.helper is None:
        parser.error("state and --helper are required for a diagnostic")
    valid = diagnose(
        args.state, args.helper.absolute(), args.seconds, args.owned_fixture, args.leaf
    )
    raise SystemExit(0 if valid else 1)


if __name__ == "__main__":
    main()
