#!/usr/bin/env python3
"""Classify the watcher's final view W against oracle walks O1 and O2, aggregate only.

Inputs (all private, under PRIVATE_DIR): O1.jsonl, O2.jsonl, W.jsonl (fdu files-view
documents), soak.jsonl (the watcher's stream: initial files view W0 then change records),
writers/*.tsv (open-writer enumerations), samples_30s.tsv, timeline_5s.tsv, start.txt,
end.txt, soak.err (/usr/bin/time -l). Output: a sanitized results JSON (no private
names) on stdout plus a private per-path classification TSV for investigation.

usage: compare.py PRIVATE_DIR ROOT_ABS > results.json
"""
import glob
import json
import os
import re
import statistics
import sys
from collections import Counter, defaultdict
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import open_atomic  # noqa: E402

PRIV = sys.argv[1]
ROOT = sys.argv[2].rstrip("/")
DEC = json.JSONDecoder()

# Row tuples: (kind, bytes, allocated, mtime_ns, files, dirs); files/dirs None for non-dirs.


def load_view(path):
    """Parse a files-view document incrementally; returns (header, {path: row_tuple})."""
    header = None
    rows = {}
    with open(path, "r", encoding="utf-8", errors="surrogateescape") as f:
        for line in f:
            if not line.strip():
                continue
            if line.startswith('{"schema"') and header is None:
                header = json.loads(line)
                continue
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
                rows[obj["path"]] = (obj["kind"], obj["bytes"], obj["allocated"], obj["mtime_ns"], obj.get("files"), obj.get("dirs"))
    return header, rows


def top(path):
    return path.split("/", 1)[0]


def depth(path):
    return path.count("/") + 1


def ext_of(path):
    name = path.rsplit("/", 1)[-1]
    if "." not in name[1:]:
        return "(none)"
    e = "." + name.rsplit(".", 1)[-1]
    return e if len(e) <= 12 and re.fullmatch(r"\.[A-Za-z0-9_-]+", e) else "(none)"


def category(path):
    """Coarse, non-identifying label: extension at depth."""
    return f"{ext_of(path)} at depth {depth(path)}"


print("loading O1", file=sys.stderr); h1, O1 = load_view(os.path.join(PRIV, "O1.jsonl"))
print("loading O2", file=sys.stderr); h2, O2 = load_view(os.path.join(PRIV, "O2.jsonl"))
print("loading W", file=sys.stderr); hw, W = load_view(os.path.join(PRIV, "W.jsonl"))
print("loading W0", file=sys.stderr); h0, W0 = load_view(os.path.join(PRIV, "soak.jsonl"))

def split(rows):
    nd, d = {}, {}
    for p, r in rows.items():
        (d if r[0] == "dir" else nd)[p] = r
    return nd, d

O1f, O1d = split(O1); O2f, O2d = split(O2); Wf, Wd = split(W); W0f, W0d = split(W0)
del O1, O2, W, W0

# ---- writer enumeration: union over the soak of paths seen open for write under ROOT
writers_seen = {}
writer_samples = 0
deleted_open = []
writer_ms = []
for fn in sorted(glob.glob(os.path.join(PRIV, "writers", "w_*.tsv"))):
    writer_samples += 1
    with open(fn, "r", encoding="utf-8", errors="surrogateescape") as f:
        for line in f:
            if line.startswith("#summary"):
                m = re.search(r"under_deleted_files=(\d+) under_deleted_alloc=(\d+)", line)
                ts = int(re.search(r"ts_ns=(\d+)", line).group(1))
                deleted_open.append((ts, int(m.group(1)), int(m.group(2))))
                writer_ms.append(float(re.search(r"total_ms=([0-9.]+)", line).group(1)))
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 12:
                continue
            ts, kind, pid, fd, flags, dev, ino, nlink, size, blocks, mtime, path = parts[:12]
            if not path.startswith(ROOT + "/"):
                continue
            rel = path[len(ROOT) + 1:]
            e = writers_seen.setdefault(rel, {"first": int(ts), "last": int(ts), "samples": 0, "deleted": False, "max_size": 0, "kinds": set()})
            e["last"] = int(ts); e["samples"] += 1; e["deleted"] |= nlink == "0"
            e["max_size"] = max(e["max_size"], int(size)); e["kinds"].add(kind)

# ---- classification of every non-directory path where W differs from O1/O2
key = lambda r: r[:4] if r else None
classes = Counter(); class_bytes = Counter(); per_class_categories = defaultdict(Counter)
private_rows = []
for p in sorted(set(O1f) | set(O2f) | set(Wf)):
    a, b, w = O1f.get(p), O2f.get(p), Wf.get(p)
    ka, kb, kw = key(a), key(b), key(w)
    if ka == kb == kw:
        continue
    writer = writers_seen.get(p)
    if ka != kb:
        cls = "concurrent (O1 != O2): " + ("W matches O1" if kw == ka else "W matches O2" if kw == kb else "W matches neither")
    else:
        if w is None:
            base = "stable miss: W lacks entry present in O1 and O2"
        elif a is None:
            base = "false positive: W has entry absent from O1 and O2"
        else:
            diff = []
            if w[1] != a[1] or w[2] != a[2]:
                diff.append("size")
            if w[3] != a[3]:
                diff.append("mtime")
            if w[0] != a[0]:
                diff.append("kind")
            base = "stable miss: W stale " + "+".join(diff)
        sub = ("held open for write during soak" + (" (seen deleted-but-open)" if writer["deleted"] else "")) if writer else "no open writer seen"
        cls = f"{base}; {sub}"
    classes[cls] += 1
    ref = a or b
    delta_alloc = (ref[2] if ref else 0) - (w[2] if w else 0)
    class_bytes[cls] += abs(delta_alloc)
    per_class_categories[cls][category(p)] += 1
    private_rows.append((cls, p, ka, kb, kw, bool(writer), delta_alloc))
with open_atomic(os.path.join(PRIV, "classification_private.tsv")) as f:
    for row in private_rows:
        f.write("\t".join(str(x) for x in row) + "\n")

# ---- totals per top-level directory: W vs O2 (directory rows carry subtree totals)
top_level = sorted({p for p in O2d if "/" not in p} | {p for p in Wd if "/" not in p})
top_table = []
for i, t in enumerate(top_level):
    o, w, o1, w0 = O2d.get(t), Wd.get(t), O1d.get(t), W0d.get(t)
    g = lambda r, k: r[k] if r else None
    top_table.append({
        "dir_index": i,
        "O2_files": g(o, 4), "O2_bytes": g(o, 1), "O2_allocated": g(o, 2),
        "W_files": g(w, 4), "W_bytes": g(w, 1), "W_allocated": g(w, 2),
        "W_minus_O2_bytes": (g(w, 1) or 0) - (g(o, 1) or 0),
        "W_minus_O2_allocated": (g(w, 2) or 0) - (g(o, 2) or 0),
        "O2_minus_O1_allocated": (g(o, 2) or 0) - (g(o1, 2) or 0),
        "O2_minus_W0_allocated": (g(o, 2) or 0) - (g(w0, 2) or 0),
        "O2_minus_W0_files": (g(o, 4) or 0) - (g(w0, 4) or 0),
    })

def totals(rows):
    return {"entries": len(rows), "files": sum(1 for r in rows.values() if r[0] == "file"),
            "bytes": sum(r[1] for r in rows.values()), "allocated": sum(r[2] for r in rows.values())}
root_totals = {"O1": totals(O1f), "O2": totals(O2f), "W": totals(Wf), "W0": totals(W0f),
               "dirs": {"O1": len(O1d), "O2": len(O2d), "W": len(Wd), "W0": len(W0d)}}

# ---- growth over the soak: O2 vs W0 (the watcher's startup baseline)
growth_by_top = Counter(); growth_by_cat = Counter(); growth_open = 0; growth_total = 0
new_files = 0; grown_files = 0; removed_bytes = 0; removed_files = 0; shrink_total = 0
for p, r in O2f.items():
    r0 = W0f.get(p)
    d = r[2] - (r0[2] if r0 else 0)
    if r0 is None:
        new_files += 1
    elif d > 0:
        grown_files += 1
    if d > 0:
        growth_total += d; growth_by_top[top(p)] += d; growth_by_cat[category(p)] += d
        if p in writers_seen:
            growth_open += d
    elif d < 0:
        shrink_total += -d
for p, r0 in W0f.items():
    if p not in O2f:
        removed_bytes += r0[2]; removed_files += 1

# ---- stream statistics from soak.jsonl
ops = Counter(); inval_by_depth = Counter(); inval_root = 0; clocks = []
change_paths_by_cat = Counter(); records_ts = []
with open(os.path.join(PRIV, "soak.jsonl"), "r", encoding="utf-8", errors="surrogateescape") as f:
    for line in f:
        if not line.startswith('{"schema": "fdu.stream/2"'):
            continue
        d = json.loads(line)
        ops[d["op"]] += 1
        clocks.append(d["clock"])
        if d["op"] == "invalidate":
            if d["path"] == "":
                inval_root += 1
            inval_by_depth[depth(d["path"]) if d["path"] else 0] += 1
        else:
            change_paths_by_cat[category(d["path"])] += 1

# ---- resident cost from the sampler, start and end records
def read_tsv(name):
    with open(os.path.join(PRIV, name)) as f:
        head = f.readline().rstrip("\n").split("\t")
        return [dict(zip(head, l.rstrip("\n").split("\t"))) for l in f if l.strip() and not l.startswith("sampler:")]

def cpu_seconds(s):
    # ps TIME as [[d-]h:]m:ss.ff
    if "-" in s:
        days, s = s.split("-"); days = int(days)
    else:
        days = 0
    parts = [float(x) for x in s.split(":")]
    while len(parts) < 3:
        parts.insert(0, 0.0)
    return days * 86400 + parts[0] * 3600 + parts[1] * 60 + parts[2]

samples = read_tsv("samples_30s.tsv")
timeline = read_tsv("timeline_5s.tsv")
start = dict(l.rstrip("\n").split("=", 1) for l in open(os.path.join(PRIV, "start.txt")) if "=" in l and not l.startswith("startup_sample"))
startup_samples = [l.rstrip("\n").split("\t") for l in open(os.path.join(PRIV, "start.txt")) if l.startswith("startup_sample")]
rss = [int(s["watcher_rss_kb"]) for s in samples if s["watcher_rss_kb"].isdigit()]
cpu = [cpu_seconds(s["watcher_cputime"]) for s in samples if s["watcher_cputime"]]
fse = [cpu_seconds(s["fseventsd_cputime"]) for s in samples if s["fseventsd_cputime"]]
ts = [int(s["ts"]) for s in samples]
wall = ts[-1] - ts[0] if len(ts) > 1 else 0
# persistence writes: distinct snapshot mtimes in the 5-s timeline
snap_writes = sorted({(int(t["snap_mtime"]), int(t["snap_size"])) for t in timeline if t["snap_mtime"] != "0"})
sizes = [s for _, s in snap_writes]
time_l = {}
for l in open(os.path.join(PRIV, "soak.err")):
    m = re.match(r"\s*(\d+)\s+(maximum resident set size|peak memory footprint|page reclaims|page faults|voluntary context switches|involuntary context switches)", l)
    if m:
        time_l[m.group(2)] = int(m.group(1))
    m = re.match(r"\s*([0-9.]+) real\s+([0-9.]+) user\s+([0-9.]+) sys", l)
    if m:
        time_l["real"], time_l["user"], time_l["sys"] = map(float, m.groups())
end = [l.rstrip("\n").split("\t", 1)[1] for l in open(os.path.join(PRIV, "end.txt")) if "\t" in l]

def q(v):
    return {"min": min(v), "median": statistics.median(v), "max": max(v), "n": len(v)} if v else None

per_sample_records = [int(s.get("records","")) for s in samples if s.get("records","").isdigit()]
rec_deltas = [b - a for a, b in zip(per_sample_records, per_sample_records[1:])]
root_inv = [int(s.get("root_invalidates","")) for s in samples if s.get("root_invalidates","").isdigit()]
cpu_deltas = [b - a for a, b in zip(cpu, cpu[1:])]
fse_deltas = [b - a for a, b in zip(fse, fse[1:])]

resident = {
    "python_wrapper_note": "the installed fdu is the Python package CLI (native extension in-process); a trivial-root watch measured 25.8 MB RSS as the wrapper baseline",
    "startup_seconds_to_initial_report": float(start.get("startup_seconds", "nan")),
    "rss_kb_after_start": int(start.get("rss_kb_after_start", 0)),
    "cputime_after_start": start.get("cputime_after_start"),
    "startup_rss_samples_kb": [int(s[2]) for s in startup_samples if len(s) > 2 and s[2].isdigit()],
    "snapshot_after_start": start.get("snapshot_after_start"),
    "lock_held_at_start": None,
    "soak_wall_seconds_sampled": wall,
    "rss_kb": q(rss),
    "rss_kb_first_last": [rss[0], rss[-1]] if rss else None,
    "threads": q([int(s["watcher_threads"]) for s in samples if s["watcher_threads"].isdigit()]),
    "watcher_cpu_seconds_total": (cpu[-1] - cpu[0]) if len(cpu) > 1 else None,
    "watcher_cpu_seconds_at_end": cpu[-1] if cpu else None,
    "watcher_cpu_utilization": ((cpu[-1] - cpu[0]) / wall) if wall else None,
    "watcher_cpu_per_30s_sample": q(cpu_deltas),
    "fseventsd_cpu_seconds_delta": (fse[-1] - fse[0]) if len(fse) > 1 else None,
    "fseventsd_cpu_per_30s_sample": q(fse_deltas),
    "load1": q([float(s["load1"]) for s in samples if s["load1"]]),
    "time_l": time_l,
    "persistence": {"writes": len(snap_writes), "snapshot_bytes": q(sizes), "writes_per_minute": (len(snap_writes) / wall * 60) if wall else None,
                     "bytes_written_total": sum(sizes)},
    "stream_records_per_30s": q(rec_deltas),
    "root_invalidations_at_end": root_inv[-1] if root_inv else None,
    "writers_enumeration_ms": q(writer_ms),
    "open_writers_under_root_per_sample": {"fds": q([int(s.get("under_fds","")) for s in samples if s.get("under_fds","").isdigit()]),
                                           "files": q([int(s.get("under_files","")) for s in samples if s.get("under_files","").isdigit()]),
                                           "pids": q([int(s.get("under_pids","")) for s in samples if s.get("under_pids","").isdigit()])},
    "end_log": end,
}

def anon_counter(c, n=12):
    return [{"category": k, "count": v} for k, v in c.most_common(n)]

result = {
    "root_entries": root_totals,
    "top_level_directories": top_table,
    "miss_classification": [{"class": k, "paths": v, "abs_allocated_delta_bytes": class_bytes[k], "top_categories": anon_counter(per_class_categories[k], 6)} for k, v in classes.most_common()],
    "writers": {
        "samples": writer_samples,
        "distinct_paths_open_for_write_under_root": len(writers_seen),
        "distinct_paths_seen_deleted_but_open": sum(1 for e in writers_seen.values() if e["deleted"]),
        "by_category": anon_counter(Counter(category(p) for p in writers_seen), 12),
        "deleted_open_files_per_sample": q([f for _, f, _ in deleted_open]),
        "deleted_open_alloc_per_sample": q([a for _, _, a in deleted_open]),
    },
    "growth_W0_to_O2": {
        "allocated_growth_bytes": growth_total,
        "allocated_growth_in_files_ever_open_for_write": growth_open,
        "share_of_growth_in_open_files": (growth_open / growth_total) if growth_total else None,
        "allocated_shrink_bytes": shrink_total,
        "new_files": new_files, "grown_existing_files": grown_files,
        "removed_files": removed_files, "removed_allocated_bytes": removed_bytes,
        "by_top_level_dir_index": [{"dir_index": top_level.index(t) if t in top_level else None, "allocated_growth": v} for t, v in growth_by_top.most_common(10)],
        "by_category": [{"category": k, "allocated_growth": v} for k, v in growth_by_cat.most_common(12)],
    },
    "stream": {
        "records_by_op": dict(ops),
        "root_invalidations": inval_root,
        "invalidations_by_depth": dict(inval_by_depth),
        "clock_first": clocks[0] if clocks else None, "clock_last": clocks[-1] if clocks else None,
        "changed_paths_by_category": anon_counter(change_paths_by_cat, 12),
    },
    "resident_cost": resident,
}
if not any("W copied" in l and "stream_bytes=" in l for l in open(os.path.join(PRIV, "end.txt"))): print(json.dumps(result, indent=1))

# ---- cross-check: W reconstructed from the stream (W0 + change records up to the W copy)
copy_bytes = None
for l in open(os.path.join(PRIV, "end.txt")):
    m = re.search(r"W copied: .*stream_bytes=(\d+)", l)
    if m:
        copy_bytes = int(m.group(1))
if copy_bytes:
    Ws = dict(W0f)
    Wsd = dict(W0d)
    applied = 0
    with open(os.path.join(PRIV, "soak.jsonl"), "rb") as f:
        data = f.read(copy_bytes)
    for line in data.decode("utf-8", errors="surrogateescape").splitlines():
        if not line.startswith('{"schema": "fdu.stream/2"'):
            continue
        d = json.loads(line)
        applied += 1
        if d["op"] == "upsert":
            row = (d["kind"], d["bytes"], d["allocated"], d["mtime_ns"], None, None)
            if d["kind"] == "dir":
                Wsd[d["path"]] = row; Ws.pop(d["path"], None)
            else:
                Ws[d["path"]] = row; Wsd.pop(d["path"], None)
        elif d["op"] == "remove":
            Ws.pop(d["path"], None); Wsd.pop(d["path"], None)
            # a removed directory takes its subtree with it
            prefix = d["path"] + "/"
            for k in [k for k in Ws if k.startswith(prefix)]:
                Ws.pop(k)
            for k in [k for k in Wsd if k.startswith(prefix)]:
                Wsd.pop(k)
    same = sum(1 for p, r in Ws.items() if key(Wf.get(p)) == key(r))
    only_stream = [p for p in Ws if p not in Wf]
    only_persisted = [p for p in Wf if p not in Ws]
    differ = [p for p in Ws if p in Wf and key(Wf[p]) != key(Ws[p])]
    result["stream_vs_persisted_W"] = {
        "records_applied_up_to_copy": applied,
        "non_dir_entries_stream": len(Ws), "non_dir_entries_persisted": len(Wf),
        "identical": same, "only_in_stream_view": len(only_stream), "only_in_persisted": len(only_persisted), "attrs_differ": len(differ),
        "differ_categories": anon_counter(Counter(category(p) for p in differ + only_stream + only_persisted), 8),
    }
    print(json.dumps(result, indent=1))
