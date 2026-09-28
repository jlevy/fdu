"""Reproducible, fixture-only FSEvents history exploration (Python stdlib)."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import stat
import subprocess
import tempfile
import time
import unittest
from pathlib import Path, PurePosixPath
from unittest.mock import patch

SCHEMA = "fdu-fsevents-probe-v2"
HISTORY_DONE = 0x10
DEGRADED = 0x01 | 0x02 | 0x04 | 0x08 | 0x20 | 0x40 | 0x80
FILE_EVENTS = 0x10
FULL_HISTORY = 0x80
IS_DIRECTORY = 0x20000
PROCESS_TIMEOUT = 20
DEEP_PATH = Path("deep/a/b/c/d/e/f/g/h/i/j/data")


class ProbeFailure(RuntimeError):
    def __init__(self, record: dict) -> None:
        super().__init__(json.dumps(record))
        self.record = record


def source_hashes() -> dict[str, str]:
    return {
        name: hashlib.sha256(Path(__file__).with_name(name).read_bytes()).hexdigest()
        for name in ["probe.c", "run.py"]
    }


def verify_provenance(saved: dict, binary: Path) -> None:
    if saved["source_sha256"] != source_hashes():
        raise ValueError("probe sources changed since preparation; use the recorded revision")
    if saved["binary_sha256"] != hashlib.sha256(binary.read_bytes()).hexdigest():
        raise ValueError("compiled helper changed since preparation; recreate the fixture")


def save(path: Path, value: object) -> None:
    """Replace a complete record atomically; interrupted writes leave no partial JSON."""
    staging = path.with_suffix(".pending")
    with staging.open("x") as output:
        json.dump(value, output, indent=2, sort_keys=True)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    staging.replace(path)


def invoke(binary: Path, *args: object) -> list[dict]:
    try:
        result = subprocess.run(
            [str(binary), *map(str, args)],
            capture_output=True,
            text=True,
            timeout=PROCESS_TIMEOUT,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        raise ProbeFailure(
            {
                "kind": "timeout",
                "seconds": PROCESS_TIMEOUT,
                "partial_stdout": (error.stdout or b"").decode(errors="replace"),
            }
        ) from error
    if result.returncode:
        raise ProbeFailure(
            {
                "kind": "helper-exit",
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }
        )
    return [json.loads(line) for line in result.stdout.splitlines()]


def scan(root: Path) -> dict:
    """Independent full oracle using lstat, including allocation and link identity."""
    result = {}
    pending = [root]
    while pending:
        path = pending.pop()
        info = path.lstat()
        key = path.relative_to(root).as_posix()
        directory = stat.S_ISDIR(info.st_mode)
        result[key] = {
            "mode": info.st_mode,
            "size": info.st_size,
            "blocks": info.st_blocks,
            "mtime_ns": info.st_mtime_ns,
            "inode": info.st_ino,
            "nlink": info.st_nlink,
            "ctime_ns": info.st_ctime_ns,
            "device": info.st_dev,
        }
        if directory:
            pending.extend(path.iterdir())
    return result


def scopes(events: list[dict], file_events: bool) -> tuple[list[str], list[str]]:
    dirty = set()
    refused = []
    for event in events:
        flags = event["flags"]
        if flags & DEGRADED:
            refused.append(f"degradation:{flags:#x}")
        if flags & HISTORY_DONE:
            continue
        if event.get("ancestor", False):
            dirty.add(".")
            continue
        if not event["inside"]:
            refused.append("outside-root")
            continue
        raw = bytes.fromhex(event["path_hex"])
        path = os.fsdecode(raw)
        parts = PurePosixPath(path).parts
        if path.startswith("/") or ".." in parts or "\0" in path:
            raise ValueError("unsafe event path")
        if file_events and not flags & IS_DIRECTORY:
            path = str(PurePosixPath(path).parent)
        dirty.add(str(PurePosixPath(path)))
    # Rescanning subtrees is deliberately conservative; measure scope inflation.
    normalized = []
    for path in sorted(dirty, key=lambda p: (len(PurePosixPath(p).parts), p)):
        if not any(
            parent == "." or path == parent or path.startswith(parent + "/")
            for parent in normalized
        ):
            normalized.append(path)
    return normalized, refused


def reconcile(root: Path, baseline: dict, dirty: list[str]) -> dict:
    candidate = baseline.copy()
    for scope in dirty:
        for key in list(candidate):
            if scope == "." or key == scope or key.startswith(scope + "/"):
                del candidate[key]
        path = root / scope
        if path.exists() or path.is_symlink():
            for key, value in scan(path).items():
                candidate[str(PurePosixPath(scope) / key)] = value
        # Parent metadata can change on child creation/deletion/rename.
        for parent in PurePosixPath(scope).parents:
            parent_path = root / parent
            if parent_path.is_dir():
                info = parent_path.lstat()
                candidate[str(parent)] = {
                    "mode": info.st_mode,
                    "size": info.st_size,
                    "blocks": info.st_blocks,
                    "mtime_ns": info.st_mtime_ns,
                    "inode": info.st_ino,
                    "nlink": info.st_nlink,
                    "ctime_ns": info.st_ctime_ns,
                    "device": info.st_dev,
                }
    return candidate


def differences(left: dict, right: dict) -> list[str]:
    return sorted(key for key in left.keys() | right.keys() if left.get(key) != right.get(key))


def prepare(directory: Path) -> None:
    sources = source_hashes()
    directory.mkdir(parents=True, exist_ok=False)
    root = directory / "fixture"
    root.mkdir()
    for name in [DEEP_PATH, Path("edit/data"), Path("delete/data"), Path("rename/data")]:
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"baseline\n")
    binary = directory / "probe"
    subprocess.run(
        [
            "xcrun",
            "clang",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-fblocks",
            "-O2",
            str(Path(__file__).with_name("probe.c")),
            "-framework",
            "CoreServices",
            "-o",
            str(binary),
        ],
        check=True,
        timeout=60,
    )
    provenance = {
        "source_sha256": sources,
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "compiler": subprocess.check_output(
            ["xcrun", "clang", "--version"], text=True
        ).splitlines()[0],
        "sdk_version": subprocess.check_output(["xcrun", "--show-sdk-version"], text=True).strip(),
    }
    verify_provenance(provenance, binary)
    # Drain initial fixture changes before capturing the pre-mutation fence.
    first = invoke(binary, "capture", root)[0]
    drained = invoke(binary, "replay", root, first["device_fence"], FULL_HISTORY)
    captured = invoke(binary, "capture", root)[0]
    captured["cursor"] = drained[-1]["latest_id"]
    baseline = scan(root)
    save(
        directory / "state.json",
        {
            "schema": SCHEMA,
            "captured": captured,
            "baseline": baseline,
            "captured_at": time.time(),
            "platform": platform.mac_ver()[0],
            "architecture": platform.machine(),
            "mutation_at": None,
            "baseline_drain": drained,
            "capture_pid": os.getpid(),
            "provenance": provenance,
        },
    )
    print(f"Prepared {directory}; no resident probe remains running.")


def mutate(directory: Path, scenario: str) -> None:
    state_path = directory / "state.json"
    state = json.loads(state_path.read_text())
    if state["mutation_at"] is not None:
        raise ValueError("fixture already mutated")
    root = directory / "fixture"
    if root.is_symlink() or differences(state["baseline"], scan(root)):
        raise ValueError("fixture differs from baseline; refusing mutations")
    with (root / DEEP_PATH).open("ab") as output:
        output.write(b"deep appended payload" * 300)
    if scenario == "mixed":
        (root / "edit/data").write_bytes(b"replacement" * 1000)
        (root / "delete/data").unlink()
        (root / "rename").rename(root / "renamed")
        (root / "new/subdir").mkdir(parents=True)
        (root / "new/subdir/data").write_bytes(b"new allocation" * 1000)
    state["mutation_at"] = time.time()
    state["mutation_pid"] = os.getpid()
    state["scenario"] = scenario
    save(state_path, state)
    print(f"Mutated fixture using {scenario} scenario; no probe stream was running.")


def replay(directory: Path, output: Path) -> None:
    state = json.loads((directory / "state.json").read_text())
    if state["schema"] != SCHEMA:
        raise ValueError("unsupported probe state")
    verify_provenance(state["provenance"], directory / "probe")
    root = directory / "fixture"
    baseline = state["baseline"]
    oracle = scan(root)
    changed = differences(baseline, oracle)
    trials = []
    failures = []
    # Both orderings reduce the chance that a delayed publication only helps FullHistory.
    for flags in [
        0,
        FULL_HISTORY,
        FILE_EVENTS,
        FILE_EVENTS | FULL_HISTORY,
        FILE_EVENTS | FULL_HISTORY,
        FILE_EVENTS,
        FULL_HISTORY,
        0,
    ]:
        try:
            records = invoke(
                directory / "probe", "replay", root, state["captured"]["cursor"], flags
            )
        except ProbeFailure as error:
            failures.append({"create_flags": flags, "error": error.record})
            continue
        identity, summary = records[0], records[-1]
        events = [record for record in records if record["type"] == "event"]
        dirty, refused = scopes(events, bool(flags & FILE_EVENTS))
        if identity["journal_uuid"] != state["captured"]["journal_uuid"]:
            refused.append("journal-uuid-changed")
        if identity["device"] != state["captured"]["device"]:
            refused.append("device-changed")
        candidate = reconcile(root, baseline, dirty)
        historical_scopes, _ = scopes(
            [e for e in events if not e["after_history_done"]], bool(flags & FILE_EVENTS)
        )
        trials.append(
            {
                "summary": summary,
                "events": events,
                "dirty_scopes": dirty,
                "refused": refused,
                "mismatch_paths": differences(candidate, oracle),
                "overlap_event_count": sum(
                    e["id"] <= state["captured"]["cursor"]
                    for e in events
                    if not e["flags"] & HISTORY_DONE
                ),
                "root_scope": "." in dirty,
                "history_only_mismatch_paths": differences(
                    reconcile(root, baseline, historical_scopes), oracle
                ),
            }
        )
    if differences(oracle, scan(root)):
        raise RuntimeError("fixture changed during oracle comparison")
    report = {
        "schema": SCHEMA,
        "platform": state["platform"],
        "architecture": state["architecture"],
        "provenance": state["provenance"],
        "filesystem": state["captured"]["filesystem"],
        "scenario": state.get("scenario", "quiet"),
        "capture_pid": state["capture_pid"],
        "mutation_pid": state.get("mutation_pid"),
        "baseline_drain_summary": state["baseline_drain"][-1],
        "device_time_fence": state["captured"]["device_fence"],
        "cursor": state["captured"]["cursor"],
        "cursor_age_seconds": time.time() - state["captured_at"],
        "mutation_age_seconds": time.time() - state["mutation_at"]
        if state["mutation_at"]
        else None,
        "changed_paths": changed,
        "trials": trials,
        "failures": failures,
        "oracle_fields": list(next(iter(oracle.values())).keys()),
        "negative_control_mismatch_paths": differences(reconcile(root, baseline, []), oracle),
        "limitations": [
            "No reboot or journal-loss injection",
            "Finite synthetic fixture",
            "No journal completeness proof",
            "No fdu engine integration or speed claim",
        ],
    }
    save(output, report)
    print(
        json.dumps(
            {
                "output": str(output),
                "changed_paths": len(changed),
                "trials": [
                    {
                        "flags": t["summary"]["create_flags"],
                        "events": len(t["events"]),
                        "scopes": t["dirty_scopes"],
                        "mismatches": t["mismatch_paths"],
                        "refused": t["refused"],
                    }
                    for t in trials
                ],
            },
            indent=2,
        )
    )
    if failures or any(t["mismatch_paths"] or t["refused"] for t in trials):
        raise SystemExit(1)


class OracleTests(unittest.TestCase):
    def test_helper_failures_preserve_diagnostics(self) -> None:
        failures = [
            subprocess.CompletedProcess([], 2, '{"type":"partial"}\n', "journal unavailable"),
            subprocess.TimeoutExpired([], PROCESS_TIMEOUT, output=b'{"type":"partial"}\n'),
        ]
        for result in failures:
            kwargs = (
                {"side_effect": result}
                if isinstance(result, Exception)
                else {"return_value": result}
            )
            with patch("subprocess.run", **kwargs), self.assertRaises(ProbeFailure) as caught:
                invoke(Path("unused"), "capture")
            self.assertIn("partial", json.dumps(caught.exception.record))
        self.assertEqual(caught.exception.record["kind"], "timeout")

    def test_changed_source_or_binary_refuses_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            binary = Path(temp) / "probe"
            binary.write_bytes(b"compiled helper")
            saved = {
                "source_sha256": source_hashes(),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            }
            verify_provenance(saved, binary)
            binary.write_bytes(b"different helper")
            with self.assertRaisesRegex(ValueError, "compiled helper changed"):
                verify_provenance(saved, binary)
            saved["source_sha256"]["probe.c"] = "different source"
            with self.assertRaisesRegex(ValueError, "sources changed"):
                verify_provenance(saved, binary)

    def test_controls_overlap_and_unsafe_paths(self) -> None:
        events = [
            {"id": 1, "flags": HISTORY_DONE, "inside": False, "path_hex": ""},
            {"id": 2, "flags": 0, "inside": True, "path_hex": b"a/b".hex()},
            {"id": 3, "flags": 0, "inside": True, "path_hex": b"a".hex()},
        ]
        self.assertEqual(scopes(events, False), (["a"], []))
        self.assertEqual(scopes(events, True), (["."], []))
        self.assertEqual(
            scopes([{**events[0], "flags": HISTORY_DONE | 2}], False), ([], ["degradation:0x12"])
        )
        for path in ["../escape", "/absolute", "a/../../escape", "bad\0path"]:
            with self.assertRaises(ValueError):
                scopes([{**events[1], "path_hex": path.encode().hex()}], False)
        with self.assertRaises(ValueError):
            scopes([{**events[1], "path_hex": "not-hex"}], False)

    def test_oracle_detects_omitted_deep_change(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "deep").mkdir()
            (root / "deep/file").write_bytes(b"old")
            baseline = scan(root)
            (root / "deep/file").write_bytes(b"changed" * 1000)
            oracle = scan(root)
            self.assertIn("deep/file", differences(reconcile(root, baseline, []), oracle))
            self.assertEqual(reconcile(root, baseline, ["deep"]), oracle)
            (root / "deep/file").unlink()
            dirty, _ = scopes([{"flags": 0, "inside": True, "path_hex": b"deep/".hex()}], False)
            self.assertEqual(reconcile(root, baseline, dirty), scan(root))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["prepare", "mutate", "replay", "self-test"])
    parser.add_argument("directory", nargs="?", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--scenario", choices=["deep", "mixed"], default="mixed")
    args = parser.parse_args()
    if args.action == "self-test":
        unittest.main(argv=[__file__])
        return
    if args.directory is None:
        parser.error("directory is required")
    directory = args.directory.resolve()
    if args.action == "prepare":
        prepare(directory)
    elif args.action == "mutate":
        mutate(directory, args.scenario)
    else:
        replay(directory, args.output or directory / "result.json")


if __name__ == "__main__":
    main()
