#!/usr/bin/env python3
"""Marking cost on a populated root, with a concurrent I/O latency probe on the same volume.

  mark_cost.py ROOT PROBE_DIR OUT.json --stage root|depth3|all

Stages (run in this order on the same fixture, under the timing lock, each a short hold):
  root    full walk twice (cold/warm oracle + timing); start probe; fsctl-mark ROOT (in-process,
          timed); GET immediately (exact? synchronous?); compare with the walk; probe latency
          before/during/after; walk timing with and without the CMNEXT attributes.
  depth3  mark every directory at relative depth <= 3 (apfs.util -M semantics via the in-process
          fsctl); count, total time, per-call distribution; GET on each origin (time distribution).
  all     mark every remaining directory; same numbers; bulk walk timing afterwards.
The probe writes/deletes 4 KiB files in PROBE_DIR (same volume, outside ROOT) and records
per-iteration latency; the report gives p50/p90/max before, during and after the mark.
"""
import json
import os
import re
import statistics
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402

DS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")
PROBE_SRC = r'''
import os, sys, time, json
d, out, secs = sys.argv[1], sys.argv[2], float(sys.argv[3])
os.makedirs(d, exist_ok=True)
lat = []
t_end = time.perf_counter() + secs
i = 0
while time.perf_counter() < t_end:
    a = time.perf_counter()
    p = os.path.join(d, "p%d" % (i % 64))
    with open(p, "wb") as fh:
        fh.write(b"p" * 4096)
    os.unlink(p)
    lat.append((time.time(), (time.perf_counter() - a) * 1000))
    i += 1
    time.sleep(0.002)
json.dump(lat, open(out, "w"))
'''


def walk(root, base_only=False, gen_only=False):
    args = [DS, "walk", root, "-o", "/dev/null"] + (["-b"] if base_only else []) + (["-g"] if gen_only else [])
    r = subprocess.run(args, capture_output=True, text=True)
    m = re.search(r"entries=(\d+) dirs=(\d+) files=(\d+) other=(\d+) pruned=\d+ wall_ms=([\d.]+) alloc=(\d+) data=(\d+)", r.stdout)
    return {"entries": int(m.group(1)), "dirs": int(m.group(2)), "files": int(m.group(3)), "wall_ms": float(m.group(5)), "alloc": int(m.group(6)), "data": int(m.group(7))}


def list_dirs(root, maxdepth=None):
    out = []
    for dp, dns, fns in os.walk(root):
        rel = os.path.relpath(dp, root)
        depth = 0 if rel == "." else rel.count(os.sep) + 1
        if maxdepth is not None and depth >= maxdepth:
            dns[:] = []
        for d in dns:
            out.append((depth + 1, os.path.join(dp, d)))
    return out


def dist(xs):
    if not xs:
        return {}
    s = sorted(xs)
    return {"n": len(s), "p50": round(statistics.median(s), 3), "p90": round(s[int(len(s) * 0.9)], 3), "max": round(s[-1], 3), "mean": round(statistics.fmean(s), 3), "sum": round(sum(s), 1)}


def mark_many(paths):
    lat = []
    fails = 0
    for p in paths:
        a = time.perf_counter()
        try:
            dsapi.mark(p, 1)
        except OSError:
            fails += 1
        lat.append((time.perf_counter() - a) * 1000)
    return {"count": len(paths), "fails": fails, "ms": dist(lat)}


def get_many(paths):
    lat = []
    for p in paths:
        lat.append(dsapi.get(p)["us"])
    return dist(lat)


def main():
    root, probe_dir, out = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2]), sys.argv[3]
    stage = sys.argv[sys.argv.index("--stage") + 1]
    log = {"stage": stage, "root_gencount_before": dsapi.gencount(root)}
    if stage == "root":
        log["walk_cold"] = walk(root)
        log["walk_warm"] = walk(root)
        log["walk_warm_base_only"] = walk(root, True)
        log["walk_warm_ext_again"] = walk(root)
        log["walk_warm_gen_only"] = walk(root, gen_only=True)
        log["walk_warm_base_again"] = walk(root, True)
        # fsctl GET on the still-unmarked root: a kernel-side recursive walk, timed once
        g = dsapi.get(root)
        log["get_on_unmarked_root"] = {"desc": g["desc"], "phys": g["phys"], "ms": round(g["us"] / 1000, 1)}
        probe_out = os.path.join(os.path.dirname(out), "probe-lat.json")
        pr = subprocess.Popen([sys.executable, "-c", PROBE_SRC, probe_dir, probe_out, "40"])
        time.sleep(8)
        t_mark0 = time.time()
        a = time.perf_counter()
        res = dsapi.mark(root, 1)
        mark_ms = (time.perf_counter() - a) * 1000
        t_mark1 = time.time()
        g0 = dsapi.get(root)
        g1 = dsapi.get(root)
        time.sleep(8)
        pr.wait()
        lat = json.load(open(probe_out))
        before = [l for t, l in lat if t < t_mark0]
        during = [l for t, l in lat if t_mark0 <= t <= t_mark1]
        after = [l for t, l in lat if t > t_mark1]
        # the gap: longest interval between consecutive probe iterations around the mark
        gaps = [(lat[i + 1][0] - lat[i][0]) * 1000 for i in range(len(lat) - 1)]
        log["mark"] = {"ms": round(mark_ms, 1), "returned": (res["gen"], res["desc"], res["phys"]),
                       "get_immediately": (g0["gen"], g0["desc"], g0["phys"], round(g0["us"], 1)), "get_second": (g1["gen"], g1["desc"], g1["phys"], round(g1["us"], 1)),
                       "exact_vs_walk": g0["desc"] == log["walk_warm"]["entries"] and g0["phys"] == log["walk_warm"]["alloc"]}
        log["probe"] = {"before_ms": dist(before), "during_ms": dist(during), "after_ms": dist(after), "max_gap_ms": round(max(gaps), 1), "iterations": len(lat)}
        log["walk_after_mark"] = walk(root)
    elif stage == "depth3":
        dirs = list_dirs(root, maxdepth=3)
        paths = [p for _, p in dirs]
        a = time.perf_counter()
        log["mark_depth3"] = mark_many(paths)
        log["mark_depth3"]["total_ms"] = round((time.perf_counter() - a) * 1000, 1)
        log["get_origins_us"] = get_many(paths)
        log["root_get"] = dsapi.get(root)
        log["walk_after"] = walk(root)
    elif stage == "all":
        dirs = list_dirs(root)
        paths = [p for _, p in dirs if dsapi.gencount(p) == 0]
        a = time.perf_counter()
        log["mark_all"] = mark_many(paths)
        log["mark_all"]["total_ms"] = round((time.perf_counter() - a) * 1000, 1)
        log["mark_all"]["already_origins"] = len(dirs) - len(paths)
        allp = [p for _, p in dirs]
        log["get_origins_us"] = get_many(allp)
        log["root_get"] = dsapi.get(root)
        log["walk_after"] = walk(root)
        log["walk_after_base_only"] = walk(root, True)
    log["root_gencount_after"] = dsapi.gencount(root)
    write_text_atomic(out, json.dumps(log, indent=1, default=str))
    print(json.dumps(log, default=str))


if __name__ == "__main__":
    main()
