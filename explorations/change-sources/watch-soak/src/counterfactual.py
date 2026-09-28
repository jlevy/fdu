#!/usr/bin/env python3
"""Counterfactual for the open-writer supplement: what an event-only watcher (no root
reconciliation) would have missed, and how much a libproc writer enumeration recovers.

The soak's watcher root-reconciled every ~20 s (every rename escalates), so its final
view had no stable misses. To size the supplement we ask instead: of the entries whose
metadata differs between the watcher's startup baseline W0 and the final walk O2, which
were held open for write during the soak (their growth produces no FSEvent until close),
and how many bytes of the tree's change did they carry? Also: how many of the stream's
upserts on those paths are explained only by reconciliation (a path never seen in a
raw FSEvents callback during the parallel 10-minute livewatch capture).

usage: counterfactual.py PRIVATE_DIR ROOT_ABS -> JSON (aggregate only)
"""
import glob
import json
import os
import re
import sys
from collections import Counter

PRIV = sys.argv[1]
ROOT = sys.argv[2].rstrip("/")
DEC = json.JSONDecoder()


def load_view(path):
    rows = {}
    with open(path, "r", encoding="utf-8", errors="surrogateescape") as f:
        for line in f:
            start = line.find('"files": [')
            if not line.startswith('{"view": "files"') or start < 0:
                continue
            i = start + len('"files": [')
            n = len(line)
            while i < n:
                while i < n and line[i] in " ,\n\t":
                    i += 1
                if i >= n or line[i] == "]":
                    break
                obj, i = DEC.raw_decode(line, i)
                if obj["kind"] != "dir":
                    rows[obj["path"]] = (obj["kind"], obj["bytes"], obj["allocated"], obj["mtime_ns"])
            break
    return rows


def ext_of(path):
    name = path.rsplit("/", 1)[-1]
    if "." not in name[1:]:
        return "(none)"
    e = "." + name.rsplit(".", 1)[-1]
    return e if len(e) <= 12 and re.fullmatch(r"\.[A-Za-z0-9_-]+", e) else "(none)"


def category(path):
    return f"{ext_of(path)} at depth {path.count('/') + 1}"


W0 = load_view(os.path.join(PRIV, "soak.jsonl"))
O2 = load_view(os.path.join(PRIV, "O2.jsonl"))

writers = {}
for fn in sorted(glob.glob(os.path.join(PRIV, "writers", "w_*.tsv"))):
    with open(fn, "r", encoding="utf-8", errors="surrogateescape") as f:
        for line in f:
            if line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 12 or not parts[11].startswith(ROOT + "/"):
                continue
            rel = parts[11][len(ROOT) + 1:]
            w = writers.setdefault(rel, {"samples": 0, "deleted": False})
            w["samples"] += 1
            w["deleted"] |= parts[7] == "0"
n_samples = len(glob.glob(os.path.join(PRIV, "writers", "w_*.tsv")))

# raw FSEvents paths seen in the 10-minute livewatch capture (relative to root)
live_paths = set()
for line in open(os.path.join(PRIV, "livewatch.jsonl")):
    d = json.loads(line)
    if d["type"] == "event" and d.get("inside"):
        live_paths.add(d["rel"])

changed = {p for p in set(W0) | set(O2) if W0.get(p) != O2.get(p)}
grown = {p for p in changed if p in W0 and p in O2 and O2[p][2] > W0[p][2]}
new = {p for p in changed if p not in W0}
gone = {p for p in changed if p not in O2}
modified_same_alloc = {p for p in changed if p in W0 and p in O2 and O2[p][2] == W0[p][2]}

def bytes_delta(paths):
    return sum((O2[p][2] if p in O2 else 0) - (W0[p][2] if p in W0 else 0) for p in paths)

open_changed = {p for p in changed if p in writers}
open_whole_soak = {p for p in open_changed if writers[p]["samples"] >= n_samples - 2}
open_grown = grown & set(writers)
categories = Counter(category(p) for p in open_changed)

# stream upserts on open paths: were they ever the subject of a raw event during the capture?
stream_upserts_open = Counter()
with open(os.path.join(PRIV, "soak.jsonl"), "r", encoding="utf-8", errors="surrogateescape") as f:
    for line in f:
        if not line.startswith('{"schema": "fdu.stream/2"'):
            continue
        d = json.loads(line)
        if d["op"] == "upsert" and d["path"] in writers:
            stream_upserts_open[d["path"]] += 1

print(json.dumps({
    "writer_samples": n_samples,
    "changed_entries_W0_to_O2": len(changed),
    "of_which_new": len(new), "of_which_removed": len(gone), "of_which_grown": len(grown), "of_which_modified_same_allocation": len(modified_same_alloc),
    "changed_entries_held_open_for_write_at_some_sample": len(open_changed),
    "changed_entries_held_open_for_nearly_the_whole_soak": len(open_whole_soak),
    "grown_entries_held_open": len(open_grown),
    "allocated_delta_all_changed_bytes": bytes_delta(changed),
    "allocated_delta_open_changed_bytes": bytes_delta(open_changed),
    "allocated_growth_grown_entries_bytes": bytes_delta(grown),
    "allocated_growth_open_grown_bytes": bytes_delta(open_grown),
    "open_changed_by_category": [{"category": k, "count": v} for k, v in categories.most_common(10)],
    "open_paths_with_stream_upserts": len(stream_upserts_open),
    "stream_upserts_on_open_paths_total": sum(stream_upserts_open.values()),
    "open_changed_paths_seen_in_raw_fsevents_capture_10min": len(open_changed & live_paths),
    "open_paths_total_seen_in_raw_capture_10min": len(set(writers) & live_paths),
    "distinct_open_paths": len(writers),
}, indent=1))
