#!/usr/bin/env python3
"""Print the ordered mark/flush/event timeline of selected trials from live-ops.jsonl."""

from __future__ import annotations

import json
import sys
import os
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import complete_lines  # noqa: E402


def main() -> None:
    path = sys.argv[1]
    wanted = set(sys.argv[2].split(",")) if len(sys.argv) > 2 else None
    reps = {int(r) for r in sys.argv[3].split(",")} if len(sys.argv) > 3 else None
    for line in complete_lines(path):
        record = json.loads(line)
        if wanted and record["op"] not in wanted:
            continue
        if reps and record.get("rep") not in reps:
            continue
        if "error" in record:
            print(f"== {record['op']} r{record['rep']}: ERROR {record['error']}")
            continue
        t0 = record["watcher"]["t"]
        print(f"== {record['op']} r{record['rep']} (global_now at start {record['watcher']['global_now']}, device_fence {record['watcher']['device_fence']})")
        items = []
        for mark in record["marks"]:
            if mark["type"] == "mark":
                items.append((mark["t"], f"MARK  {mark['label']}"))
            else:
                items.append((mark["t"], f"FLUSH {mark['flush_ms']:.2f} ms, events so far {mark['events_so_far']}"))
        for event in record["events"]:
            items.append((event["t"], f"EVENT id={event['id']} {event['rel'] or '<dir>'} {event['names']}"))
        for entry in record["log"]:
            if entry["label"] == "replay":
                rp = entry["replay"]
                summary = rp.get("summary") or {}
                evs = "; ".join(f"{e['rel'] or '<dir>'}:{e['names']}(id={e['id']})" for e in rp["events"]) or "none"
                items.append((t0 + entry["t"] + 1e-6, f"REPLAY[{entry['mark']}] cursor={rp['cursor']} rc={rp['returncode']} history_done={summary.get('history_done')} latest={summary.get('latest_id')} wall={rp['wall_seconds']}s -> {evs}"))
        items.sort()
        for t, text in items:
            print(f"  {t - t0:8.3f}  {text}")
        for sample in record["stats"]:
            st = sample["stat"]
            print(f"  STAT {sample['label']:<30} {st}")
        for key in ("avail_delta_after_orphan_growth", "avail_delta_after_close", "prealloc_bytesalloc", "synchronous", "checkpoint", "child_returncode"):
            if key in record:
                print(f"  {key} = {record[key]}")


if __name__ == "__main__":
    main()
