"""Metadata-only, cross-process FSEvents experiment over an existing directory.

The monitored root is never mutated. State and all temporary files belong on an
explicit external scratch volume. This measures a Python prototype, not fdu.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import platform
import re
import stat
import subprocess
import sys
import tempfile
import time
import unittest
from collections.abc import Generator
from dataclasses import asdict, dataclass
from pathlib import Path, PurePosixPath
from typing import Any, NamedTuple, cast
from unittest.mock import patch

# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from scripts.atomic_write import open_atomic  # noqa: E402

SCHEMA = "fdu-real-tree-replay-v1"
FILE_EVENTS = 0x10
FULL_HISTORY = 0x80
HISTORY_DONE = 0x10
IS_DIRECTORY = 0x20000
IS_FILE = 0x10000
IS_SYMLINK = 0x40000
MUST_SCAN_SUBDIRS = 0x01
GLOBAL_DEGRADATION = 0x02 | 0x04 | 0x08 | 0x20 | 0x40 | 0x80
DIRECTORY_LIFECYCLE = 0x100 | 0x200 | 0x800 | 0x400000
ALLOWED_FLAGS = (0, FILE_EVENTS, FULL_HISTORY, FILE_EVENTS | FULL_HISTORY)
HELPER_TIMEOUT_SECONDS = 20
OPEN_DIRECTORY = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
BLOCK_BYTES = 512
SAFE_LABEL = re.compile(r"[A-Za-z0-9_-]{1,64}\Z")
GIB = 1024**3
MIB = 1024**2
DEFAULT_MAX_ENTRIES = 1_000_000


@dataclass(frozen=True)
class Limits:
    """Explicit research budgets, independent of production fdu policy."""

    max_entries: int = DEFAULT_MAX_ENTRIES
    max_state_bytes: int = 2 * GIB
    reserve_free_bytes: int = 2 * GIB

    def validate(self) -> None:
        if min(self.max_entries, self.max_state_bytes, self.reserve_free_bytes) <= 0:
            raise ValueError("resource budgets must be positive")


class EntryBudgetExceeded(RuntimeError):
    pass


@dataclass(frozen=True)
class ScopePlan:
    """Shallow listings do not cover descendant changes; recursive scans do."""

    relist: tuple[str, ...] = ()
    recursive: tuple[str, ...] = ()
    fallback_reasons: tuple[str, ...] = ()


class Facts(NamedTuple):
    mode: int
    size: int
    blocks: int
    mtime_ns: int
    ctime_ns: int
    device: int
    inode: int
    nlink: int


Inventory = dict[str, Facts]
Record = dict[str, Any]


def facts(value: os.stat_result) -> Facts:
    return Facts(
        value.st_mode,
        value.st_size,
        value.st_blocks,
        value.st_mtime_ns,
        value.st_ctime_ns,
        value.st_dev,
        value.st_ino,
        value.st_nlink,
    )


def identity(value: Facts) -> tuple[int, int]:
    return value.device, value.inode


def normalized_parts(path: str) -> tuple[str, ...]:
    parts = PurePosixPath(path).parts
    if path.startswith("/") or ".." in parts or "\0" in path:
        raise ValueError("unsafe relative scope")
    return tuple(part for part in parts if part != ".")


@contextlib.contextmanager
def directory_fd(path: Path) -> Generator[int]:
    """Open every ancestor without following symlinks, including the root itself."""
    if not path.is_absolute():
        raise ValueError("absolute directory path required")
    descriptor = os.open("/", OPEN_DIRECTORY)
    try:
        for component in path.parts[1:]:
            if component in (".", ".."):
                raise ValueError("noncanonical directory component")
            child = os.open(component, OPEN_DIRECTORY, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        yield descriptor
    finally:
        os.close(descriptor)


def error_record(path: str, error: OSError | ValueError) -> Record:
    return {
        "path": path,
        "kind": type(error).__name__,
        "errno": error.errno if isinstance(error, OSError) else None,
        "message": str(error),
    }


def scan_fd(
    descriptor: int,
    prefix: str,
    device: int,
    max_entries: int = DEFAULT_MAX_ENTRIES,
    *,
    recursive: bool = True,
) -> tuple[Inventory, Record]:
    """Observe metadata only; reject mount crossings and directory-substitution races."""
    inventory: Inventory = {}
    stats: Record = {"observed_entries": 0, "directory_lists": 0, "errors": []}

    def visit(current: int, name: str, expected: Facts | None = None) -> None:
        if len(inventory) >= max_entries:
            raise EntryBudgetExceeded("metadata entry budget exhausted")
        observed = facts(os.fstat(current))
        if observed.device != device:
            raise ValueError("cross-device directory")
        if expected is not None and identity(observed) != identity(expected):
            raise ValueError("directory identity changed between observation and open")
        inventory[name] = observed
        stats["observed_entries"] += 1
        stats["directory_lists"] += 1
        with os.scandir(current) as children:
            for entry in children:
                relative = str(PurePosixPath(name) / entry.name)
                try:
                    metadata = facts(entry.stat(follow_symlinks=False))
                    if metadata.device != device:
                        raise ValueError("cross-device entry")
                    if recursive and stat.S_ISDIR(metadata.mode):
                        child = os.open(entry.name, OPEN_DIRECTORY, dir_fd=current)
                        try:
                            visit(child, relative, metadata)
                        finally:
                            os.close(child)
                    else:
                        if len(inventory) >= max_entries:
                            raise EntryBudgetExceeded("metadata entry budget exhausted")
                        inventory[relative] = metadata
                        stats["observed_entries"] += 1
                except (OSError, ValueError) as error:
                    stats["errors"].append(error_record(relative, error))

    try:
        visit(descriptor, prefix)
    except (OSError, ValueError) as error:
        stats["errors"].append(error_record(prefix, error))
    return inventory, stats


def full_scan(
    root: Path, expected: tuple[int, int], max_entries: int = DEFAULT_MAX_ENTRIES
) -> tuple[Inventory, Record]:
    start = time.perf_counter()
    with directory_fd(root) as descriptor:
        observed = facts(os.fstat(descriptor))
        if identity(observed) != expected:
            raise ValueError("root identity changed")
        inventory, stats = scan_fd(descriptor, ".", observed.device, max_entries)
    stats["seconds"] = time.perf_counter() - start
    return inventory, stats


def normalize_events(records: list[Record], flags: int) -> ScopePlan:
    relist: set[str] = set()
    recursive: set[str] = set()
    fallback: set[str] = set()
    for event in records:
        if event.get("type") != "event":
            continue
        event_flags = event["flags"]
        if event_flags & GLOBAL_DEGRADATION:
            fallback.add(f"global-degradation:{event_flags:#x}")
        if event_flags & HISTORY_DONE:
            if event_flags & MUST_SCAN_SUBDIRS:
                fallback.add("unscoped-recursive-control")
            continue
        if event.get("ancestor"):
            recursive.add(".")
            continue
        if not event.get("inside"):
            fallback.add("unclassified-outside-root-event")
            continue
        path = os.fsdecode(bytes.fromhex(event["path_hex"]))
        normalized_parts(path)
        path = str(PurePosixPath(path))
        kind = event_flags & (IS_FILE | IS_DIRECTORY | IS_SYMLINK)
        if flags & FILE_EVENTS and kind not in (IS_FILE, IS_DIRECTORY, IS_SYMLINK):
            fallback.add("ambiguous-item-kind")
            continue
        if event_flags & MUST_SCAN_SUBDIRS:
            recursive.add(path if kind in (0, IS_DIRECTORY) else str(PurePosixPath(path).parent))
            continue
        if not flags & FILE_EVENTS:
            relist.add(path)
        elif kind == IS_DIRECTORY:
            if event_flags & DIRECTORY_LIFECYCLE:
                relist.add(str(PurePosixPath(path).parent))
                recursive.add(path)
            else:
                relist.add(path)
        else:
            relist.add(str(PurePosixPath(path).parent))
    if fallback:
        return ScopePlan(recursive=(".",), fallback_reasons=tuple(sorted(fallback)))
    reduced_recursive: list[str] = []
    for path in sorted(recursive, key=lambda value: (len(PurePosixPath(value).parts), value)):
        if not any(contains(parent, path) for parent in reduced_recursive):
            reduced_recursive.append(path)
    # In particular, a shallow '.' must not absorb a separately nominated deep directory.
    reduced_relist = tuple(
        sorted(
            path
            for path in relist
            if not any(contains(parent, path) for parent in reduced_recursive)
        )
    )
    return ScopePlan(reduced_relist, tuple(reduced_recursive))


def contains(parent: str, path: str) -> bool:
    return parent == "." or path == parent or path.startswith(parent + "/")


@contextlib.contextmanager
def scope_fd(
    root_fd: int, scope: str, device: int, baseline: Inventory
) -> Generator[tuple[int, str, Inventory]]:
    """Widen missing scopes to their nearest extant parent; never traverse a symlink."""
    current = os.dup(root_fd)
    current_path = "."
    ancestors: Inventory = {".": facts(os.fstat(current))}
    try:
        for part in normalized_parts(scope):
            try:
                observed = facts(os.stat(part, dir_fd=current, follow_symlinks=False))
            except FileNotFoundError:
                break
            if observed.device != device:
                raise ValueError("cross-device scope")
            if stat.S_ISLNK(observed.mode):
                raise ValueError("symlink in event scope; refusing traversal")
            if not stat.S_ISDIR(observed.mode):
                break
            child = os.open(part, OPEN_DIRECTORY, dir_fd=current)
            if identity(facts(os.fstat(child))) != identity(observed):
                os.close(child)
                raise ValueError("scope ancestor changed before open")
            os.close(current)
            current = child
            current_path = str(PurePosixPath(current_path) / part)
            ancestors[current_path] = facts(os.fstat(current))
            previous = baseline.get(current_path)
            if previous is None or identity(previous) != identity(ancestors[current_path]):
                # A replaced ancestor invalidates all its cached descendants, not just scope.
                break
        yield current, current_path, ancestors
    finally:
        os.close(current)


def reconcile(
    root: Path,
    baseline: Inventory,
    expected: tuple[int, int],
    plan: ScopePlan,
    max_entries: int = DEFAULT_MAX_ENTRIES,
) -> tuple[Inventory, Record]:
    candidate = baseline.copy()
    stats: Record = {
        "observed_entries": 0,
        "directory_lists": 0,
        "ancestor_observations": 0,
        "effective_scopes": [],
        "relisted_scopes": [],
        "recursive_scopes": [],
        "hardlink_alias_scopes": [],
        "removed_entries": 0,
        "hardlink_index_seconds": 0.0,
        "fallback_reasons": list(plan.fallback_reasons),
        "errors": [],
    }
    completed_recursive: set[str] = set()
    completed_relist: set[str] = set()
    alias_parents: dict[tuple[int, int], set[str]] | None = None
    index_started = time.perf_counter()
    child_index: dict[str, set[str]] = {}
    for path in candidate:
        if path != ".":
            child_index.setdefault(str(PurePosixPath(path).parent), set()).add(path)
    stats["parent_index_seconds"] = time.perf_counter() - index_started

    def accumulate(scan_stats: Record) -> None:
        stats["observed_entries"] += scan_stats["observed_entries"]
        stats["directory_lists"] += scan_stats["directory_lists"]
        stats["errors"].extend(scan_stats["errors"])

    def remember_aliases(
        previous: Facts | None, current: Facts | None, aliases: set[tuple[int, int]]
    ) -> None:
        if previous == current:
            return
        for value in (previous, current):
            if value is not None and stat.S_ISREG(value.mode) and value.nlink > 1:
                aliases.add(identity(value))

    with directory_fd(root) as root_fd:
        root_facts = facts(os.fstat(root_fd))
        if identity(root_facts) != expected:
            raise ValueError("root identity changed")
        work = [(path, False) for path in plan.relist] + [(path, True) for path in plan.recursive]
        for scope, requested_recursive in work:
            try:
                with scope_fd(root_fd, scope, root_facts.device, candidate) as (
                    descriptor,
                    effective,
                    ancestors,
                ):
                    if any(contains(prior, effective) for prior in completed_recursive):
                        continue
                    recursive = requested_recursive and effective == scope
                    old_directory = candidate.get(effective)
                    if (
                        old_directory is None
                        or not stat.S_ISDIR(old_directory.mode)
                        or identity(old_directory) != identity(ancestors[effective])
                    ):
                        recursive = True
                    if not recursive and effective in completed_relist:
                        continue
                    observed, scan_stats = scan_fd(
                        descriptor, effective, root_facts.device, max_entries, recursive=recursive
                    )
                    accumulate(scan_stats)
                    stats["ancestor_observations"] += len(ancestors)
                    stats["effective_scopes"].append(effective)
                    if scan_stats["errors"]:
                        continue
                    remove: set[str] = {effective} if recursive else set()
                    discovered: list[str] = []
                    if not recursive:
                        direct_before = {
                            path: candidate[path] for path in child_index.get(effective, set())
                        }
                        remove.update(path for path in direct_before if path not in observed)
                        failed = False
                        for path, value in list(observed.items()):
                            if path == effective:
                                continue
                            previous = direct_before.get(path)
                            was_directory = previous is not None and stat.S_ISDIR(previous.mode)
                            is_directory = stat.S_ISDIR(value.mode)
                            if was_directory and not is_directory:
                                remove.add(path)
                            if is_directory and (
                                not was_directory
                                or previous is None
                                or identity(previous) != identity(value)
                            ):
                                remove.add(path)
                                child = os.open(
                                    PurePosixPath(path).name, OPEN_DIRECTORY, dir_fd=descriptor
                                )
                                try:
                                    if identity(facts(os.fstat(child))) != identity(value):
                                        raise ValueError(
                                            "new directory changed before recursive discovery"
                                        )
                                    subtree, subtree_stats = scan_fd(
                                        child, path, root_facts.device, max_entries
                                    )
                                finally:
                                    os.close(child)
                                accumulate(subtree_stats)
                                if subtree_stats["errors"]:
                                    failed = True
                                    break
                                observed.update(subtree)
                                discovered.append(path)
                        if failed:
                            continue
                    # A changed inode can have retained names outside these dirty scopes.
                    # Index all baseline files lazily: creating a second link changes a
                    # formerly single-link inode too, so filtering the index by nlink is unsafe.
                    touched_aliases: set[tuple[int, int]] = set()

                    for path, current in observed.items():
                        remember_aliases(candidate.get(path), current, touched_aliases)
                    pending_removal = list(remove)
                    while pending_removal:
                        path = pending_removal.pop()
                        pending_removal.extend(child_index.pop(path, ()))
                        previous = candidate.pop(path, None)
                        if previous is not None:
                            stats["removed_entries"] += 1
                            remember_aliases(previous, observed.get(path), touched_aliases)
                        if path != ".":
                            child_index.get(str(PurePosixPath(path).parent), set()).discard(path)
                    if touched_aliases:
                        if alias_parents is None:
                            alias_started = time.perf_counter()
                            alias_parents = {}
                            for path, value in baseline.items():
                                if stat.S_ISREG(value.mode):
                                    alias_parents.setdefault(identity(value), set()).add(
                                        str(PurePosixPath(path).parent)
                                    )
                            stats["hardlink_index_seconds"] = time.perf_counter() - alias_started
                        queued = {path for path, _ in work}
                        for file_identity in sorted(touched_aliases):
                            for parent in sorted(alias_parents.get(file_identity, set[str]())):
                                if parent not in queued:
                                    work.append((parent, False))
                                    queued.add(parent)
                                    stats["hardlink_alias_scopes"].append(parent)
                    candidate.update(ancestors)
                    candidate.update(observed)
                    for path in ancestors.keys() | observed.keys():
                        if path != ".":
                            child_index.setdefault(str(PurePosixPath(path).parent), set()).add(path)
                    if len(candidate) > max_entries:
                        raise EntryBudgetExceeded("candidate entry budget exhausted")
                    if recursive:
                        completed_recursive.add(effective)
                        stats["recursive_scopes"].append(effective)
                    else:
                        completed_relist.add(effective)
                        stats["relisted_scopes"].append(effective)
                    completed_recursive.update(discovered)
                    stats["recursive_scopes"].extend(discovered)
            except (OSError, ValueError) as error:
                stats["errors"].append(error_record(scope, error))
    stats["root_fallback"] = "." in stats["recursive_scopes"]
    stats["root_relisted"] = "." in stats["relisted_scopes"]
    return candidate, stats


def rollups(inventory: Inventory) -> dict[str, dict[str, int]]:
    """Count regular-file bytes per path, with no hard-link/clone deduplication."""
    totals: dict[str, dict[str, int]] = {}
    for path in sorted(inventory, key=lambda key: len(PurePosixPath(key).parts), reverse=True):
        value = inventory[path]
        row = totals.setdefault(
            path,
            {
                "entries": 0,
                "directories": 0,
                "regular_files": 0,
                "symlinks": 0,
                "apparent_bytes": 0,
                "allocated_bytes": 0,
            },
        )
        row["entries"] += 1
        if stat.S_ISREG(value.mode):
            row["regular_files"] += 1
            row["apparent_bytes"] += value.size
            row["allocated_bytes"] += value.blocks * BLOCK_BYTES
        elif stat.S_ISDIR(value.mode):
            row["directories"] += 1
        elif stat.S_ISLNK(value.mode):
            row["symlinks"] += 1
        if path != ".":
            parent = str(PurePosixPath(path).parent)
            parent_row = totals.setdefault(parent, {key: 0 for key in row})
            for key, count in row.items():
                parent_row[key] += count
            if not stat.S_ISDIR(value.mode):
                del totals[path]
    return totals


def compare_oracles(candidate: Inventory, before: Inventory, after: Inventory) -> Record:
    paths = candidate.keys() | before.keys() | after.keys()
    raw_before = sorted(path for path in paths if candidate.get(path) != before.get(path))
    raw_after = sorted(path for path in paths if candidate.get(path) != after.get(path))
    concurrent = sorted(path for path in paths if before.get(path) != after.get(path))
    stable_misses = sorted(
        path for path in paths if before.get(path) == after.get(path) != candidate.get(path)
    )
    unresolved = sorted(
        path
        for path in concurrent
        if candidate.get(path) not in (before.get(path), after.get(path))
    )
    return {
        "raw_before_mismatches": raw_before,
        "raw_after_mismatches": raw_after,
        "concurrent_paths": concurrent,
        "stable_mismatches": stable_misses,
        "unresolved_concurrent_mismatches": unresolved,
        "stable_equal_count": len(paths) - len(concurrent) - len(stable_misses),
        "status": "mismatch" if stable_misses else "inconclusive" if concurrent else "equal",
    }


def state_bytes(state: Path) -> int:
    return sum(
        (Path(parent) / name).lstat().st_size
        for parent, _, files in os.walk(state, followlinks=False)
        for name in files
    )


def check_space(state: Path, limits: Limits) -> int:
    remaining = limits.max_state_bytes - state_bytes(state)
    filesystem = os.statvfs(state)
    free = filesystem.f_bavail * filesystem.f_frsize
    if remaining <= 0 or free <= limits.reserve_free_bytes:
        raise OSError(28, "external scratch state budget or free-space reserve exhausted")
    return remaining


def atomic_json(path: Path, value: object, budget: tuple[Path, Limits] | None = None) -> None:
    """Write only inside private state; fsync and replace never modify the baseline in-place."""
    with open_atomic(path) as output:
        # Owner-only, as this private state has always been written.
        os.fchmod(output.fileno(), 0o600)
        remaining = check_space(*budget) if budget else None
        written = 0
        since_space_check = 0
        for chunk in json.JSONEncoder(separators=(",", ":")).iterencode(value):
            length = len(chunk.encode("utf-8"))
            written += length
            since_space_check += length
            if remaining is not None and written + 1 > remaining:
                raise OSError(28, "state byte budget exhausted during serialization")
            if budget and since_space_check >= MIB:
                output.flush()
                check_space(*budget)
                since_space_check = 0
            output.write(chunk)
        output.write("\n")
    with directory_fd(path.parent) as parent_fd:
        os.fsync(parent_fd)


def load_inventory(value: Record) -> Inventory:
    if value["fields"] != list(Facts._fields):
        raise ValueError("unsupported metadata fields")
    result: Inventory = {}
    for path, row in value["inventory"].items():
        normalized_parts(path)
        if len(row) != len(Facts._fields) or any(type(item) is not int for item in row):
            raise ValueError("malformed inventory row")
        result[path] = Facts(*row)
    return result


def source_hash() -> str:
    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def helper_provenance(helper: Path) -> Record:
    saved = json.loads(helper.with_name("state.json").read_text())["provenance"]
    actual_binary = hashlib.sha256(helper.read_bytes()).hexdigest()
    actual_source = hashlib.sha256(Path(__file__).with_name("probe.c").read_bytes()).hexdigest()
    if (
        saved["binary_sha256"] != actual_binary
        or saved["source_sha256"]["probe.c"] != actual_source
    ):
        raise ValueError("native helper differs from its recorded build provenance")
    return {"helper_path": str(helper), "build": saved, "harness_sha256": source_hash()}


def check_provenance(saved: Record) -> Path:
    helper = Path(saved["helper_path"])
    if helper_provenance(helper) != saved:
        raise ValueError("helper or harness changed since baseline capture")
    return helper


def native(helper: Path, *args: object) -> list[Record]:
    result = subprocess.run(
        [str(helper), *map(str, args)],
        capture_output=True,
        text=True,
        timeout=HELPER_TIMEOUT_SECONDS,
        check=False,
    )
    if result.returncode:
        raise RuntimeError(
            f"native helper exited {result.returncode}: {result.stdout}\n{result.stderr}"
        )
    records = [json.loads(line) for line in result.stdout.splitlines()]
    if not records or records[0].get("type") != "identity":
        raise ValueError("native helper returned no identity")
    if args[0] == "replay" and (
        records[-1].get("type") != "summary"
        or not records[-1].get("history_done")
        or records[-1].get("timeout")
        or records[-1].get("truncated")
    ):
        raise ValueError("native helper returned incomplete replay")
    return records


def validate_locations(root: Path, state: Path) -> None:
    if not root.is_absolute() or not state.is_absolute():
        raise ValueError("root and state must be absolute paths")
    with directory_fd(root):
        pass
    if state.is_relative_to(root) or root.is_relative_to(state):
        raise ValueError("state and monitored root must be disjoint")
    if len(state.parts) < 4 or state.parts[1] != "Volumes":
        raise ValueError("state must be on an explicitly mounted external volume")
    volume = Path(*state.parts[:3])
    if not os.path.ismount(volume):
        raise ValueError("external state volume is not mounted")
    parent = state if state.exists() else state.parent
    # Case-insensitive aliases must not make a state directory inside ROOT look disjoint.
    root_id = root_identity(root)
    for ancestor in (parent, *parent.parents):
        if root_identity(ancestor) == root_id:
            raise ValueError("state resolves inside monitored root")
    with directory_fd(parent) as descriptor:
        if os.fstat(descriptor).st_dev == os.stat(Path.home()).st_dev:
            raise ValueError("state resolves to the internal home volume")


def root_identity(root: Path) -> tuple[int, int]:
    with directory_fd(root) as descriptor:
        return identity(facts(os.fstat(descriptor)))


def capture_baseline(root: Path, state: Path, alias: str, helper: Path, limits: Limits) -> None:
    validate_locations(root, state)
    if not SAFE_LABEL.fullmatch(alias):
        raise ValueError("alias must be a short filename-free label")
    if state.exists():
        raise ValueError("baseline state directory already exists")
    provenance = helper_provenance(helper)
    limits.validate()
    state.mkdir(mode=0o700)
    budget = state, limits
    check_space(*budget)
    started_at = time.time()
    expected = root_identity(root)
    captured = native(helper, "capture", root)[0]
    if captured["device"] != expected[0] or not captured["device_fence"]:
        raise ValueError("invalid pre-scan device fence")
    inventory, scan_stats = full_scan(root, expected, limits.max_entries)
    if scan_stats["errors"] or root_identity(root) != expected:
        atomic_json(state / "baseline-failure.json", {"scan": scan_stats}, budget)
        raise ValueError("baseline scan incomplete; no baseline published")
    rollup_start = time.perf_counter()
    totals = rollups(inventory)
    rollup_seconds = time.perf_counter() - rollup_start
    baseline: Record = {
        "schema": SCHEMA,
        "root": str(root),
        "alias": alias,
        "identity": expected,
        "captured": captured,
        "provenance": provenance,
        "started_at": started_at,
        "completed_at": time.time(),
        "platform": platform.mac_ver()[0],
        "architecture": platform.machine(),
        "fields": list(Facts._fields),
        "inventory": inventory,
        "rollups": totals,
        "scan": scan_stats,
        "rollup_seconds": rollup_seconds,
        "limits": asdict(limits),
    }
    save_start = time.perf_counter()
    atomic_json(state / "baseline.json", baseline, budget)
    summary = {
        "schema": SCHEMA,
        "alias": alias,
        "operation": "baseline",
        "root_totals": totals["."],
        "scan_seconds": scan_stats["seconds"],
        "rollup_seconds": rollup_seconds,
        "save_seconds": time.perf_counter() - save_start,
        "status": "captured",
        "provenance": public_provenance(provenance),
        "capture_started_at": started_at,
        "capture_completed_at": baseline["completed_at"],
        "resource_limits": asdict(limits),
    }
    atomic_json(state / "baseline-summary.json", summary, budget)
    print(json.dumps(summary, indent=2))


def public_provenance(provenance: Record) -> Record:
    return {
        "harness_sha256": provenance["harness_sha256"],
        "binary_sha256": provenance["build"]["binary_sha256"],
        "native_source_sha256": provenance["build"]["source_sha256"]["probe.c"],
        "compiler": provenance["build"]["compiler"],
        "sdk_version": provenance["build"]["sdk_version"],
    }


def refresh(root: Path, state: Path, label: str, flags: int) -> int:
    refresh_started_at = time.time()
    preflight_start = time.perf_counter()
    validate_locations(root, state)
    if not SAFE_LABEL.fullmatch(label) or flags not in ALLOWED_FLAGS:
        raise ValueError("invalid trial label or create flags")
    trial = state / label
    trial.mkdir(mode=0o700)
    phase = "baseline-load"
    budget: tuple[Path, Limits] | None = None
    try:
        baseline_data = json.loads((state / "baseline.json").read_text())
        if baseline_data["schema"] != SCHEMA or baseline_data["root"] != str(root):
            raise ValueError("baseline schema or root mismatch")
        expected = (int(baseline_data["identity"][0]), int(baseline_data["identity"][1]))
        limits = Limits(**baseline_data["limits"])
        limits.validate()
        budget = state, limits
        check_space(*budget)
        helper = check_provenance(baseline_data["provenance"])
        if root_identity(root) != expected:
            raise ValueError("root identity changed")
        timings: dict[str, float] = {"preflight": time.perf_counter() - preflight_start}
        phase = "oracle-before"
        before, before_stats = full_scan(root, expected, limits.max_entries)
        before_totals = rollups(before)
        atomic_json(
            trial / "oracle-before.json",
            {"fields": list(Facts._fields), "inventory": before, "scan": before_stats},
            budget,
        )
        # Reload within the timed path; the oracle never supplies candidate metadata.
        incremental_start = time.perf_counter()
        phase = "load"
        start = time.perf_counter()
        baseline_data = json.loads((state / "baseline.json").read_text())
        baseline = load_inventory(baseline_data)
        if len(baseline) > limits.max_entries:
            raise EntryBudgetExceeded("baseline entry budget exhausted")
        timings["load"] = time.perf_counter() - start
        phase = "replay"
        start = time.perf_counter()
        records = native(helper, "replay", root, baseline_data["captured"]["device_fence"], flags)
        timings["replay"] = time.perf_counter() - start
        if records[0]["journal_uuid"] != baseline_data["captured"]["journal_uuid"]:
            raise ValueError("journal database changed")
        if records[0]["device"] != expected[0] or root_identity(root) != expected:
            raise ValueError("root or device changed during replay")
        phase = "normalization"
        start = time.perf_counter()
        plan = normalize_events(records, flags)
        timings["normalization"] = time.perf_counter() - start
        phase = "scoped-scan"
        start = time.perf_counter()
        candidate, candidate_stats = reconcile(root, baseline, expected, plan, limits.max_entries)
        timings["scoped_scan"] = time.perf_counter() - start
        phase = "rollup-and-diff"
        start = time.perf_counter()
        candidate_totals = rollups(candidate)
        changes = sorted(
            path
            for path in baseline.keys() | candidate.keys()
            if baseline.get(path) != candidate.get(path)
        )
        timings["rollup_and_diff"] = time.perf_counter() - start
        phase = "candidate-save"
        start = time.perf_counter()
        atomic_json(
            trial / "candidate.json",
            {
                "fields": list(Facts._fields),
                "inventory": candidate,
                "rollups": candidate_totals,
                "scan": candidate_stats,
                "changed_paths": changes,
                "cursor_advanced": False,
            },
            budget,
        )
        timings["candidate_save"] = time.perf_counter() - start
        timings["instrumented_refresh_segment"] = time.perf_counter() - incremental_start
        # Candidate publication precedes this independent full traversal.
        phase = "oracle-after"
        full_control_start = time.perf_counter()
        after, after_stats = full_scan(root, expected, limits.max_entries)
        after_rollup_start = time.perf_counter()
        after_totals = rollups(after)
        after_stats["rollup_seconds"] = time.perf_counter() - after_rollup_start
        after_save_start = time.perf_counter()
        atomic_json(
            trial / "oracle-after.json",
            {
                "fields": list(Facts._fields),
                "inventory": after,
                "rollups": after_totals,
                "scan": after_stats,
            },
            budget,
        )
        after_stats["save_seconds"] = time.perf_counter() - after_save_start
        after_stats["full_control_seconds"] = time.perf_counter() - full_control_start
        comparison = compare_oracles(candidate, before, after)
        errors = before_stats["errors"] + candidate_stats["errors"] + after_stats["errors"]
        if errors:
            comparison["status"] = "inconclusive"
        private = {
            "comparison": comparison,
            "native_records": records,
            "scope_plan": asdict(plan),
            "candidate_stats": candidate_stats,
            "before_stats": before_stats,
            "after_stats": after_stats,
        }
        atomic_json(trial / "details-private.json", private, budget)
        summary = {
            "schema": SCHEMA,
            "alias": baseline_data["alias"],
            "operation": "refresh",
            "status": comparison["status"],
            "create_flags": flags,
            "provenance": public_provenance(baseline_data["provenance"]),
            "filesystem": baseline_data["captured"]["filesystem"],
            "platform": baseline_data["platform"],
            "architecture": baseline_data["architecture"],
            "timings_seconds": timings,
            "oracle_before_seconds": before_stats["seconds"],
            "oracle_after_seconds": after_stats["seconds"],
            "full_control_seconds": after_stats["full_control_seconds"],
            "full_control_rollup_seconds": after_stats["rollup_seconds"],
            "full_control_save_seconds": after_stats["save_seconds"],
            "oracle_totals_before": before_totals.get("."),
            "oracle_totals_after": after_totals.get("."),
            "oracle_before_observed_entries": before_stats["observed_entries"],
            "oracle_after_observed_entries": after_stats["observed_entries"],
            "baseline_capture_started_at": baseline_data["started_at"],
            "baseline_capture_completed_at": baseline_data["completed_at"],
            "refresh_started_at": refresh_started_at,
            "refresh_completed_at": time.time(),
            "gap_since_baseline_completion_seconds": refresh_started_at
            - baseline_data["completed_at"],
            "root_totals_before": baseline_data["rollups"]["."],
            "root_totals_candidate": candidate_totals["."],
            "changed_entry_count": len(changes),
            "dirty_scope_count": len(plan.relist) + len(plan.recursive),
            "planned_relist_scope_count": len(plan.relist),
            "planned_recursive_scope_count": len(plan.recursive),
            "fallback_reasons": list(plan.fallback_reasons),
            "effective_scope_count": len(candidate_stats["effective_scopes"]),
            "relisted_scope_count": len(candidate_stats["relisted_scopes"]),
            "recursive_scope_count": len(candidate_stats["recursive_scopes"]),
            "hardlink_alias_scope_count": len(candidate_stats["hardlink_alias_scopes"]),
            "candidate_observed_entries": candidate_stats["observed_entries"],
            "candidate_ancestor_observations": candidate_stats["ancestor_observations"],
            "candidate_directory_lists": candidate_stats["directory_lists"],
            "candidate_removed_entries": candidate_stats["removed_entries"],
            "parent_index_seconds": candidate_stats["parent_index_seconds"],
            "hardlink_index_seconds": candidate_stats["hardlink_index_seconds"],
            "root_fallback": candidate_stats["root_fallback"],
            "root_relisted": candidate_stats["root_relisted"],
            "error_count": len(errors),
            "mismatch_counts": {
                key: len(cast(list[object], value))
                for key, value in comparison.items()
                if isinstance(value, list)
            },
            "stable_equal_count": comparison["stable_equal_count"],
            "event_count": sum(record["type"] == "event" for record in records),
            "overlap_event_count": sum(
                record["type"] == "event"
                and not record["flags"] & HISTORY_DONE
                and record["id"] <= baseline_data["captured"]["device_fence"]
                for record in records
            ),
            "cursor_advanced": False,
            "oracle_before_warms_metadata_cache": True,
            "preflight_warms_baseline_load": True,
            "resource_limits": asdict(limits),
            "state_bytes_before_summary": state_bytes(state),
            "limits": [
                "Python prototype; full-image loading, copying, rollups, and saving",
                "Regular-file bytes per path; no hard-link or clone deduplication",
                "Concurrent paths are unverified; two matching observations are not atomicity proof",
                "Helper timings and filesystem scans are not fdu end-to-end performance",
                "Timed refresh segment excludes preflight, both oracles, and validation/reporting",
                "Full control is scan plus rollups plus save; ordered after candidate, not paired/interleaved",
            ],
        }
        atomic_json(trial / "summary-shareable.json", summary, budget)
        print(json.dumps(summary, indent=2))
        return (
            0 if comparison["status"] == "equal" else 1 if comparison["status"] == "mismatch" else 2
        )
    except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired) as error:
        summary: Record = {
            "schema": SCHEMA,
            "operation": "refresh",
            "status": "failed",
            "phase": phase,
            "error_kind": type(error).__name__,
        }
        try:
            atomic_json(
                trial / "failure-private.json",
                {"phase": phase, "kind": type(error).__name__, "message": str(error)},
                budget,
            )
            atomic_json(trial / "summary-shareable.json", summary, budget)
        except OSError:
            summary["failure_record_persisted"] = False
        print(json.dumps(summary, indent=2))
        return 2


class InstrumentTests(unittest.TestCase):
    @staticmethod
    def event(path: str, flags: int, **extra: object) -> Record:
        return {
            "type": "event",
            "path_hex": os.fsencode(path).hex(),
            "flags": flags,
            "inside": True,
            "id": 1,
            **extra,
        }

    def test_shallow_root_preserves_deep_scope_and_negative_control(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "deep").mkdir()
            (root / "untouched").mkdir()
            for name in ("root-file", "deep/file", "untouched/file"):
                (root / name).write_bytes(b"old")
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "root-file").write_bytes(b"root changed")
            (root / "deep/file").write_bytes(b"nested changed")
            records = [
                self.event("root-file", IS_FILE | 0x1000),
                self.event("deep/file", IS_FILE | 0x1000),
            ]
            plan = normalize_events(records + records, FILE_EVENTS | FULL_HISTORY)
            self.assertEqual(plan, ScopePlan(relist=(".", "deep")))
            oracle, _ = full_scan(root, expected)
            omitted, omitted_stats = reconcile(root, baseline, expected, ScopePlan(relist=(".",)))
            self.assertEqual(
                compare_oracles(omitted, oracle, oracle)["stable_mismatches"], ["deep/file"]
            )
            self.assertEqual(omitted_stats["directory_lists"], 1)
            candidate, stats = reconcile(root, baseline, expected, plan)
            self.assertEqual(candidate, oracle)
            self.assertEqual(stats["directory_lists"], 2)
            self.assertEqual(stats["observed_entries"], 6)
            self.assertTrue(stats["root_relisted"])
            self.assertFalse(stats["root_fallback"])
            self.assertEqual(stats["recursive_scopes"], [])

    def test_normalization_controls_kinds_and_overlap(self) -> None:
        flags = FILE_EVENTS | FULL_HISTORY
        self.assertEqual(
            normalize_events([self.event("link", IS_SYMLINK)], flags), ScopePlan(relist=(".",))
        )
        self.assertEqual(
            normalize_events([self.event("deep", IS_DIRECTORY)], flags), ScopePlan(relist=("deep",))
        )
        for lifecycle in (0x100, 0x200, 0x800, 0x400000):
            self.assertEqual(
                normalize_events([self.event("deep", IS_DIRECTORY | lifecycle)], flags),
                ScopePlan(relist=(".",), recursive=("deep",)),
            )
        must_scan = self.event("deep", IS_DIRECTORY | MUST_SCAN_SUBDIRS)
        self.assertEqual(
            normalize_events([must_scan, self.event("deep/nested/file", IS_FILE)], flags),
            ScopePlan(recursive=("deep",)),
        )
        for event in (
            self.event(".", 0x02, inside=False),
            self.event(".", HISTORY_DONE | 0x04, inside=False),
            self.event("file", 0x1000),
            self.event("file", MUST_SCAN_SUBDIRS),
            self.event("file", IS_FILE | IS_DIRECTORY),
        ):
            with self.subTest(event_flags=event["flags"]):
                plan = normalize_events([event], flags)
                self.assertEqual(plan.recursive, (".",))
                self.assertTrue(plan.fallback_reasons)
        self.assertEqual(
            normalize_events([self.event("deep", 0x1000)], FULL_HISTORY),
            ScopePlan(relist=("deep",)),
        )
        self.assertEqual(
            normalize_events(
                [self.event(".", MUST_SCAN_SUBDIRS, inside=False, ancestor=True)], flags
            ),
            ScopePlan(recursive=(".",)),
        )
        with self.assertRaises(ValueError):
            normalize_events([self.event("../escape", IS_FILE)], flags)

    def test_directory_lifecycle_and_in_place_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "root"
            root.mkdir()
            for directory in ("deleted", "renamed", "replaced/sub", "untouched"):
                (root / directory).mkdir(parents=True)
                (root / directory / "file").write_bytes(b"old")
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "deleted/file").unlink()
            (root / "deleted").rmdir()
            (root / "renamed").rename(root / "new-name")
            (root / "replaced").rename(base / "outside-root")
            (root / "replaced/sub").mkdir(parents=True)
            (root / "replaced/sub/new-file").write_bytes(b"replacement")
            (root / "replaced/new-sibling").mkdir()
            (root / "replaced/new-sibling/file").write_bytes(b"discover sibling too")
            (root / "created/nested").mkdir(parents=True)
            (root / "created/nested/file").write_bytes(b"new")
            candidate, stats = reconcile(root, baseline, expected, ScopePlan(relist=(".",)))
            oracle, _ = full_scan(root, expected)
            self.assertEqual(candidate, oracle)
            self.assertFalse(stats["root_fallback"])
            self.assertNotIn("untouched", stats["recursive_scopes"])
            self.assertNotIn("deleted/file", candidate)
            self.assertNotIn("renamed/file", candidate)
            # Reaching into a replaced ancestor must rediscover its other descendants.
            partial, partial_stats = reconcile(
                root, baseline, expected, ScopePlan(relist=("replaced/sub",))
            )
            self.assertEqual(partial_stats["recursive_scopes"], ["replaced"])
            self.assertIn("replaced/new-sibling/file", partial)
            self.assertNotIn("replaced/sub/file", partial)

    def test_recursive_invalidation_and_missing_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "deep/sub").mkdir(parents=True)
            (root / "deep/sub/file").write_bytes(b"before")
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "deep/sub/file").write_bytes(b"after")
            plan = normalize_events(
                [self.event("deep", MUST_SCAN_SUBDIRS | IS_DIRECTORY)], FILE_EVENTS
            )
            candidate, stats = reconcile(root, baseline, expected, plan)
            oracle, _ = full_scan(root, expected)
            self.assertEqual(candidate, oracle)
            self.assertEqual(stats["recursive_scopes"], ["deep"])
            fallback = normalize_events([self.event(".", 0x02, inside=False)], FILE_EVENTS)
            candidate, stats = reconcile(root, baseline, expected, fallback)
            self.assertEqual(candidate, oracle)
            self.assertTrue(stats["root_fallback"])
            self.assertEqual(stats["observed_entries"], len(oracle))
            (root / "deep/sub/file").unlink()
            (root / "deep/sub").rmdir()
            candidate, stats = reconcile(
                root, baseline, expected, ScopePlan(recursive=("deep/sub",))
            )
            oracle, _ = full_scan(root, expected)
            self.assertEqual(candidate, oracle)
            self.assertEqual(stats["relisted_scopes"], ["deep"])

    def test_hardlink_alias_parents_reconciled_for_change_create_and_delete(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for directory in ("left", "right"):
                (root / directory).mkdir()
            left, right = root / "left/file", root / "right/alias"
            left.write_bytes(b"before")
            expected = root_identity(root)
            for mutation in ("create", "change", "delete"):
                with self.subTest(mutation=mutation):
                    baseline, _ = full_scan(root, expected)
                    if mutation == "create":
                        os.link(left, right)
                        nominated = "right"
                    elif mutation == "change":
                        left.write_bytes(b"new data" * 100)
                        nominated = "left"
                    else:
                        left.unlink()
                        nominated = "left"
                    plan = (
                        ScopePlan(recursive=(nominated,))
                        if mutation == "change"
                        else ScopePlan(relist=(nominated,))
                    )
                    candidate, stats = reconcile(root, baseline, expected, plan)
                    oracle, _ = full_scan(root, expected)
                    self.assertEqual(candidate, oracle)
                    self.assertEqual(
                        stats["hardlink_alias_scopes"],
                        ["left" if nominated == "right" else "right"],
                    )
                    self.assertFalse(stats["root_fallback"])

    def test_narrow_change_never_scans_unchanged_sibling(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "changed").mkdir()
            (root / "untouched").mkdir()
            (root / "changed/file").write_bytes(b"before")
            (root / "untouched/file").write_bytes(b"unchanged")
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "changed/file").write_bytes(b"after" * 500)
            candidate, stats = reconcile(root, baseline, expected, ScopePlan(relist=("changed",)))
            oracle, _ = full_scan(root, expected)
            self.assertEqual(candidate, oracle)
            self.assertEqual(stats["effective_scopes"], ["changed"])
            self.assertEqual(stats["observed_entries"], 2)
            self.assertFalse(stats["root_fallback"])

    def test_entry_and_serialization_budgets_fail_without_replacing_baseline(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "file").write_bytes(b"payload")
            with self.assertRaises(EntryBudgetExceeded):
                full_scan(root, root_identity(root), max_entries=1)
            destination = root / "baseline.json"
            atomic_json(destination, {"original": True})
            original = destination.read_bytes()
            limits = Limits(max_state_bytes=state_bytes(root) + 10, reserve_free_bytes=1)
            with self.assertRaises(OSError):
                atomic_json(destination, {"large": "x" * 1000}, (root, limits))
            self.assertEqual(destination.read_bytes(), original)

    def test_changed_scopes_and_negative_control(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "root"
            root.mkdir()
            (root / "deep").mkdir()
            (root / "deep/edited").write_bytes(b"before")
            (root / "deep/deleted").write_bytes(b"delete")
            (root / "old").mkdir()
            (root / "old/item").write_bytes(b"rename")
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "deep/edited").write_bytes(b"after" * 2000)
            (root / "deep/deleted").unlink()
            (root / "deep/created").write_bytes(b"new")
            (root / "old").rename(root / "new")
            before, _ = full_scan(root, expected)
            omitted, _ = reconcile(root, baseline, expected, ScopePlan())
            self.assertEqual(compare_oracles(omitted, before, before)["status"], "mismatch")
            candidate, stats = reconcile(root, baseline, expected, ScopePlan(relist=(".", "deep")))
            after, _ = full_scan(root, expected)
            self.assertEqual(candidate, after)
            self.assertFalse(stats["errors"])
            self.assertEqual(rollups(candidate)["."]["regular_files"], 3)
            self.assertNotIn("deep/deleted", candidate)

    def test_symlink_scope_and_ancestor_substitution(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "root"
            outside = base / "outside"
            root.mkdir()
            outside.mkdir()
            (outside / "secret").write_bytes(b"never inspect")
            (root / "a").mkdir()
            (root / "a/sub").mkdir()
            expected = root_identity(root)
            baseline, _ = full_scan(root, expected)
            (root / "a").rename(root / "old-a")
            (root / "a").symlink_to(outside, target_is_directory=True)
            candidate, stats = reconcile(root, baseline, expected, ScopePlan(relist=("a/sub",)))
            self.assertTrue(stats["errors"])
            self.assertNotIn("a/secret", candidate)
            full, full_stats = full_scan(root, expected)
            self.assertFalse(full_stats["errors"])
            self.assertTrue(stat.S_ISLNK(full["a"].mode))
            self.assertNotIn("a/secret", full)
            with self.assertRaises(OSError), directory_fd(root / "a"):
                pass
            with self.assertRaises(ValueError):
                normalized_parts("../outside")

    def test_concurrent_oracles_do_not_hide_stable_misses(self) -> None:
        one = Facts(stat.S_IFREG, 1, 1, 1, 1, 1, 1, 1)
        two = one._replace(size=2)
        before = {"stable": one, "active": one, "removed": one}
        after = {"stable": one, "active": two, "created": one}
        result = compare_oracles({"stable": two, "active": one}, before, after)
        self.assertEqual(result["stable_mismatches"], ["stable"])
        self.assertEqual(result["concurrent_paths"], ["active", "created", "removed"])
        self.assertEqual(result["status"], "mismatch")
        result = compare_oracles(before, before, after)
        self.assertEqual(result["status"], "inconclusive")
        self.assertEqual(result["stable_equal_count"], 1)

    def test_atomic_failure_preserves_previous_baseline(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "baseline.json"
            atomic_json(path, {"baseline": "original"})
            original = path.read_bytes()
            with (
                patch("os.replace", side_effect=OSError(28, "no space")),
                self.assertRaises(OSError),
            ):
                atomic_json(path, {"baseline": "replacement"})
            self.assertEqual(path.read_bytes(), original)
            self.assertEqual(list(path.parent.iterdir()), [path])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    baseline = actions.add_parser("baseline")
    baseline.add_argument("root", type=Path)
    baseline.add_argument("state", type=Path)
    baseline.add_argument("--alias", required=True)
    baseline.add_argument("--helper", type=Path, required=True)
    baseline.add_argument("--max-entries", type=int, default=DEFAULT_MAX_ENTRIES)
    baseline.add_argument("--max-state-bytes", type=int, default=2 * GIB)
    baseline.add_argument("--reserve-free-bytes", type=int, default=2 * GIB)
    refresh_parser = actions.add_parser("refresh")
    refresh_parser.add_argument("root", type=Path)
    refresh_parser.add_argument("state", type=Path)
    refresh_parser.add_argument("--label", required=True)
    refresh_parser.add_argument(
        "--flags", type=int, choices=ALLOWED_FLAGS, default=FILE_EVENTS | FULL_HISTORY
    )
    actions.add_parser("self-test")
    args = parser.parse_args()
    if args.action == "self-test":
        unittest.main(argv=[__file__])
        return 0
    root, state = args.root.absolute(), args.state.absolute()
    if args.action == "baseline":
        capture_baseline(
            root,
            state,
            args.alias,
            args.helper.absolute(),
            Limits(args.max_entries, args.max_state_bytes, args.reserve_free_bytes),
        )
        return 0
    return refresh(root, state, args.label, args.flags)


if __name__ == "__main__":
    raise SystemExit(main())
