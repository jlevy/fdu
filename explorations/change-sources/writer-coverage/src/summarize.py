#!/usr/bin/env python3
"""Reduce live-ops.jsonl into the operation x {live, close, replay, stat} table."""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from typing import Any
import os
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import complete_lines  # noqa: E402

FLAG_NAMES = {
    0x1: "MustScanSubDirs", 0x2: "UserDropped", 0x4: "KernelDropped", 0x8: "EventIdsWrapped",
    0x10: "HistoryDone", 0x20: "RootChanged", 0x40: "Mount", 0x80: "Unmount",
    0x100: "ItemCreated", 0x200: "ItemRemoved", 0x400: "ItemInodeMetaMod", 0x800: "ItemRenamed",
    0x1000: "ItemModified", 0x2000: "ItemFinderInfoMod", 0x4000: "ItemChangeOwner",
    0x8000: "ItemXattrMod", 0x10000: "ItemIsFile", 0x20000: "ItemIsDir", 0x40000: "ItemIsSymlink",
    0x80000: "OwnEvent", 0x100000: "ItemIsHardlink", 0x200000: "ItemIsLastHardlink", 0x400000: "ItemCloned",
}


def flag_names(flags: int) -> str:
    names = [name for bit, name in FLAG_NAMES.items() if flags & bit]
    return "|".join(names) or "0"


TARGETS = {"sqlite_wal": ("target.db", "target.db-wal", "target.db-shm")}


def mark_time(record: dict[str, Any], label: str) -> float | None:
    for mark in record["marks"]:
        if mark.get("type") == "mark" and mark.get("label") == label:
            return mark["t"]
    return None


def events_between(record: dict[str, Any], start: float | None, end: float | None) -> list[dict[str, Any]]:
    out = []
    for event in record["events"]:
        if start is not None and event["t"] < start:
            continue
        if end is not None and event["t"] > end:
            continue
        out.append(event)
    return out


def describe(events: list[dict[str, Any]]) -> str:
    if not events:
        return "none"
    parts = []
    for event in events:
        parts.append(f"{event['rel'] or '<dir>'}:{event['names']}")
    return "; ".join(parts)


def replay_events(record: dict[str, Any], label: str) -> tuple[list[dict[str, Any]] | None, dict[str, Any] | None]:
    for entry in record["log"]:
        if entry["label"] == "replay" and entry["mark"] == label:
            return entry["replay"]["events"], entry["replay"]
    return None, None


def stat_by_label(record: dict[str, Any], label: str) -> dict[str, Any] | None:
    for sample in record["stats"]:
        if sample["label"] == label:
            return sample["stat"]
    return None


def stat_delta(before: dict[str, Any] | None, after: dict[str, Any] | None) -> str:
    if not before or not after or "error" in before or "error" in after:
        return "n/a"
    changed = [key for key in ("size", "blocks", "mtime_ns", "ctime_ns", "nlink") if before.get(key) != after.get(key)]
    return ",".join(changed) or "unchanged"


def main() -> None:
    rows: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for line in complete_lines(sys.argv[1]):
        record = json.loads(line)
        rows[record["op"]].append(record)
    print("| op | rep | live events while open (after FlushSync) | events at/after close | replay while open | replay after close | stat while open (vs before) | notes |")
    print("| --- | ---: | --- | --- | --- | --- | --- | --- |")
    for op, records in rows.items():
        for record in records:
            if "error" in record:
                print(f"| {op} | {record['rep']} | ERROR {record['error']} | | | | | |")
                continue
            start = mark_time(record, "op-start")
            close = mark_time(record, "close") or mark_time(record, "close-last") or mark_time(record, "munmap") or mark_time(record, "close-fd-mapping-alive")
            if op in ("mmap_fd_open", "mmap_fd_closed"):
                close = mark_time(record, "close-fd-mapping-alive") or mark_time(record, "munmap")
            if op == "dup_close":
                close = mark_time(record, "close-last")
            live = events_between(record, start, close)
            after = events_between(record, close, None)
            open_replay, open_summary = replay_events(record, "open-mid" if op not in ("mmap_fd_open", "mmap_fd_closed", "sqlite_wal", "rename_over_open", "unlink_open", "second_writer_append", "dup_close") else {"mmap_fd_open": "mapped-mid", "mmap_fd_closed": "mapped-mid", "sqlite_wal": "inserted-mid", "rename_over_open": "orphaned-mid", "unlink_open": "orphaned-mid", "second_writer_append": "second-closed-first-open-mid", "dup_close": "dup-alive-mid"}[op])
            close_replay, close_summary = replay_events(record, "after-close")
            before = stat_by_label(record, "before")
            held = None
            for sample in record["stats"]:
                if sample["label"].endswith(f"+{record['hold_seconds']:g}s") and ":" not in sample["label"]:
                    held = sample["stat"]
            notes = []
            for key in ("avail_delta_after_orphan_growth", "avail_delta_after_close", "prealloc_bytesalloc", "synchronous", "checkpoint"):
                if key in record:
                    notes.append(f"{key}={record[key]}")
            if open_summary and open_summary.get("returncode") != 0:
                notes.append(f"open replay rc={open_summary.get('returncode')}")
            if close_summary and close_summary.get("returncode") != 0:
                notes.append(f"close replay rc={close_summary.get('returncode')}")
            print(f"| {op} | {record['rep']} | {describe(live)} | {describe(after)} | {describe(open_replay or [])} | {describe(close_replay or [])} | {stat_delta(before, held)} | {'; '.join(notes)} |")
    # Per-op stat detail for the ctime question
    print()
    print("Stat samples (size/blocks/mtime/ctime relative to 'before'):")
    for op, records in rows.items():
        for record in records:
            if "error" in record:
                continue
            before = stat_by_label(record, "before")
            for sample in record["stats"]:
                st = sample["stat"]
                if not before or "error" in st or "error" in before:
                    continue
                d = {k: (st.get(k) - before.get(k)) if isinstance(st.get(k), int) and isinstance(before.get(k), int) else None for k in ("size", "blocks", "nlink")}
                dm = (st["mtime_ns"] - before["mtime_ns"]) / 1e9 if "mtime_ns" in st else None
                dc = (st["ctime_ns"] - before["ctime_ns"]) / 1e9 if "ctime_ns" in st else None
                print(f"  {op} r{record['rep']} {sample['label']:<28} dsize={d['size']!s:>8} dblocks={d['blocks']!s:>6} nlink={st.get('nlink')} dmtime={dm if dm is None else round(dm, 3)} dctime={dc if dc is None else round(dc, 3)}")


if __name__ == "__main__":
    main()


def per_path(events: list[dict[str, Any]]) -> dict[str, tuple[int, int]]:
    """fseventsd merges same-path events into one record carrying the latest id and OR'd flags."""
    out: dict[str, tuple[int, int]] = {}
    for event in events:
        rel = event["rel"]
        prev = out.get(rel, (0, 0))
        out[rel] = (max(prev[0], event["id"]), prev[1] | event["flags"])
    return out


def agree(live: dict[str, tuple[int, int]], replay: dict[str, tuple[int, int]] | None) -> tuple[bool, str]:
    if replay is None:
        return False, "no replay"
    problems = []
    for rel, (live_id, live_flags) in live.items():
        if rel not in replay:
            problems.append(f"{rel or '<dir>'} missing from replay")
            continue
        replay_id, replay_flags = replay[rel]
        if replay_id != live_id:
            problems.append(f"{rel or '<dir>'} id live={live_id} replay={replay_id}")
        if live_flags & ~replay_flags:
            problems.append(f"{rel or '<dir>'} live flags {flag_names(live_flags & ~replay_flags)} absent from replay")
    for rel in replay:
        if rel not in live:
            problems.append(f"{rel or '<dir>'} only in replay")
    return not problems, "; ".join(problems)


def agreement(path: str) -> None:
    """Live stream vs short-cursor replay over the same window, compared per path."""
    print()
    print("Live vs replay agreement (per path: replay id == latest live id, replay flags include all live flags; cursor = global counter at live-stream start; while-open comparison bounded by the replay's HistoryDone id):")
    print("| op | rep | paths live before close | replay while open agrees | paths live total | replay after close agrees | replay wall s (open/close) |")
    print("| --- | ---: | ---: | --- | ---: | --- | --- |")
    totals = {"open_agree": 0, "open_total": 0, "close_agree": 0, "close_total": 0}
    for line in complete_lines(path):
        record = json.loads(line)
        if "error" in record:
            continue
        op = record["op"]
        close = mark_time(record, "close") or mark_time(record, "munmap")
        if op in ("mmap_fd_open", "mmap_fd_closed"):
            close = mark_time(record, "munmap")
        if op == "dup_close":
            close = mark_time(record, "close-last")
        live_all = per_path(record["events"])
        open_map, open_wall, close_map, close_wall, open_latest = None, None, None, None, None
        for entry in record["log"]:
            if entry["label"] != "replay":
                continue
            mapping = per_path(entry["replay"]["events"])
            summary = entry["replay"].get("summary") or {}
            if entry["mark"] == "after-close":
                close_map, close_wall = mapping, entry["replay"]["wall_seconds"]
            else:
                open_map, open_wall, open_latest = mapping, entry["replay"]["wall_seconds"], summary.get("latest_id")
        # Clock-free bound: every live event whose id is at or below the while-open replay's HistoryDone
        # id existed when that replay completed and must appear in it.
        live_before = per_path([e for e in record["events"] if open_latest is not None and e["id"] <= open_latest])
        open_ok, open_why = agree(live_before, open_map)
        close_ok, close_why = agree(live_all, close_map)
        totals["open_total"] += 1
        totals["close_total"] += 1
        totals["open_agree"] += open_ok
        totals["close_agree"] += close_ok
        print(f"| {op} | {record['rep']} | {len(live_before)} | {'yes' if open_ok else 'NO: ' + open_why} | {len(live_all)} | {'yes' if close_ok else 'NO: ' + close_why} | {open_wall}/{close_wall} |")
    print(f"\nagreement: while-open {totals['open_agree']}/{totals['open_total']}, after-close {totals['close_agree']}/{totals['close_total']}")


if __name__ == "__main__" and len(sys.argv) > 2 and sys.argv[2] == "--agreement":
    agreement(sys.argv[1])
