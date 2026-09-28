#!/usr/bin/env python3
"""Aggregate a livewatch JSONL capture: raw FSEvents rate and flag mix under the root.

usage: livewatch_stats.py livewatch.jsonl  -> JSON on stdout (no paths)
"""
import json
import sys
from collections import Counter

FLAGS = {
    0x1: "MustScanSubDirs", 0x2: "UserDropped", 0x4: "KernelDropped", 0x100: "ItemCreated",
    0x200: "ItemRemoved", 0x400: "InodeMetaMod", 0x800: "ItemRenamed", 0x1000: "ItemModified",
    0x2000: "FinderInfoMod", 0x4000: "ChangeOwner", 0x8000: "XattrMod", 0x10000: "IsFile",
    0x20000: "IsDir", 0x40000: "IsSymlink", 0x100000: "IsHardlink", 0x400000: "ItemCloned",
}
events = []
ready = None
for line in open(sys.argv[1]):
    d = json.loads(line)
    if d["type"] == "ready":
        ready = d
    elif d["type"] == "event":
        events.append(d)
if not events:
    print(json.dumps({"events": 0}))
    sys.exit()
t0, t1 = ready["t"] if ready else events[0]["t"], events[-1]["t"]
flag_counts = Counter()
combo = Counter()
renamed = 0
depths = Counter()
callbacks = set()
per_minute = Counter()
paths = set()
for e in events:
    f = e["flags"]
    names = [n for b, n in FLAGS.items() if f & b]
    for n in names:
        flag_counts[n] += 1
    combo["|".join(n for n in names if n not in ("IsFile", "IsDir", "IsSymlink"))] += 1
    if f & 0x800:
        renamed += 1
    depths[e["rel"].count("/") + 1 if e["rel"] else 0] += 1
    callbacks.add(e["callback"])
    per_minute[int((e["t"] - t0) // 60)] += 1
    paths.add(e["rel"])
span = t1 - t0
print(json.dumps({
    "span_seconds": round(span, 1),
    "events": len(events),
    "events_per_minute": round(len(events) / span * 60, 1) if span else None,
    "callbacks": len(callbacks),
    "distinct_paths": len(paths),
    "events_with_ItemRenamed": renamed,
    "share_with_ItemRenamed": round(renamed / len(events), 3),
    "flag_counts": dict(flag_counts.most_common()),
    "top_flag_combinations": [{"flags": k, "count": v} for k, v in combo.most_common(10)],
    "events_by_depth": dict(sorted(depths.items())),
    "events_per_minute_series": [per_minute[i] for i in range(int(span // 60) + 1)],
}, indent=1))
