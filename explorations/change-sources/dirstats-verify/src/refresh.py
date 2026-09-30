#!/usr/bin/env python3
"""Refresh-algorithm experiment: checkpoint (gencount, totals) per origin, an agent-like
mutation workload, then (i) a gencount-pruned walk and (ii) a totals-only diff, each
compared with a full-walk oracle diff.

  refresh.py ROOT OUTSIDE_DIR OUT.json LABEL

ROOT is a marked fixture (origins wherever the previous stage left them). OUTSIDE_DIR is a
writable directory outside ROOT on the same volume (for the hard link from outside).
LABEL names the origin layout (depth3 | all). Work dirs are suffixed with LABEL so the
script can run once per layout on the same fixture. Runs under the timing lock.

Oracle diff classes: added, removed, size_changed (alloc or datalen), mtime_only.
Miss = an oracle-changed entry whose path (or, for removed entries, whose parent) was not
visited by the pruned walk.
"""
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import open_atomic, write_text_atomic  # noqa: E402

DS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")


def walk(root, outfile, ckpt=None):
    args = [DS, "walk", root, "-o", outfile] + (["-p", ckpt] if ckpt else [])
    a = time.perf_counter()
    r = subprocess.run(args, capture_output=True, text=True)
    ms = (time.perf_counter() - a) * 1000
    return r.stdout.strip(), ms


def load(tsv, root):
    """relpath -> record. kind P = pruned origin (listed, not descended)."""
    recs = {}
    rl = len(root) + 1
    with open(tsv) as fh:
        for line in fh:
            f = line.rstrip("\n").split("\t")
            kind, path = f[0], f[1][rl:]
            recs[path] = {"kind": kind, "fileid": int(f[2]), "datalen": int(f[4]), "alloc": int(f[6]), "gencount": int(f[9]), "mtime": int(f[13])}
    return recs


def parent(p):
    return p.rsplit("/", 1)[0] if "/" in p else ""


def subtree_alloc(recs, prefix):
    """Sum of alloc over regular files under prefix (each hard link name counted, like the walker)."""
    pre = prefix + "/" if prefix else ""
    return sum(r["alloc"] for p, r in recs.items() if r["kind"] == "f" and p.startswith(pre))


def cleanup(root, outside, label):
    """Remove leftovers of a previous run with the same label (keeps the fixture usable)."""
    for p in (os.path.join(root, "target", "debug", f"build-{label}"),):
        if os.path.isdir(p):
            shutil.rmtree(p)
    ren = os.path.join(root, "src", f"m-renamed-{label}")
    if os.path.isdir(ren):
        os.rename(ren, os.path.join(root, "src", f"m-restored-{label}"))
    for p in (os.path.join(root, "src", f"session-{label}.log"), os.path.join(root, "src", f"state-{label}.db"),
              os.path.join(root, "src", f"state-{label}.db-wal"), os.path.join(root, "src", f"state-{label}.db-shm"),
              os.path.join(root, "target", "debug", "deps", f"clone-{label}.rlib"), os.path.join(root, "wide", f"linked-from-outside-{label}.bin"),
              os.path.join(outside, f"outside-{label}.bin"), os.path.join(outside, f"alias-{label}.json")):
        if os.path.lexists(p):
            os.unlink(p)


def workload(root, outside, label, held):
    ev = {}
    a = time.perf_counter()
    # a. build dir with thousands of files
    b = os.path.join(root, "target", "debug", f"build-{label}")
    for d in range(60):
        dd = os.path.join(b, f"unit{d:03d}"); os.makedirs(dd)
        for f in range(50):
            with open(os.path.join(dd, f"o{f:03d}.o"), "wb") as fh:
                fh.write(b"o" * (4096 + (f % 4) * 4096))
    ev["build_files"] = 3000
    # b. log held open, grown without close
    lp = os.path.join(root, "src", f"session-{label}.log")
    fd = os.open(lp, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
    for _ in range(64):
        os.write(fd, b"L" * 32768)
    held["log_fd"] = fd
    ev["log_bytes"] = 64 * 32768
    # c. sqlite WAL held open
    c = sqlite3.connect(os.path.join(root, "src", f"state-{label}.db"))
    c.execute("pragma journal_mode=wal"); c.execute("create table t(x blob)")
    for i in range(200):
        c.execute("insert into t values (?)", (b"s" * 8192,))
    c.commit(); held["db"] = c
    # d. delete a subtree
    pkgs = sorted(x for x in os.listdir(os.path.join(root, "node_modules")) if x.startswith("pkg-"))
    shutil.rmtree(os.path.join(root, "node_modules", pkgs[len(pkgs) // 2 + (0 if label == "depth3" else 1)]))
    # e. rename a subtree
    ms = sorted(x for x in os.listdir(os.path.join(root, "src")) if os.path.isdir(os.path.join(root, "src", x)))
    os.rename(os.path.join(root, "src", ms[-1 if (label == "depth3" or len(ms) < 2) else -2]), os.path.join(root, "src", f"m-renamed-{label}"))
    # f. clone
    deps = os.path.join(root, "target", "debug", "deps")
    src = sorted(x for x in os.listdir(deps) if x.startswith("lib00001-"))[0]
    subprocess.run(["cp", "-c", os.path.join(deps, src), os.path.join(deps, f"clone-{label}.rlib")], check=True)
    # g. hard link from outside into the tree, and a write through an outside link of an inside file
    of = os.path.join(outside, f"outside-{label}.bin")
    with open(of, "wb") as fh:
        fh.write(b"O" * 65536)
    os.link(of, os.path.join(root, "wide", f"linked-from-outside-{label}.bin"))
    alias = os.path.join(outside, f"alias-{label}.json")
    os.link(os.path.join(root, "wide", "w000001.json"), alias)
    with open(alias, "ab") as fh:
        fh.write(b"A" * 16384)
    # h. appends deep in the trees
    for t in range(3):
        d = os.path.join(root, f"deep-{t}")
        for _ in range(5 + t):
            if not os.path.isdir(os.path.join(d, "d1")):
                break
            d = os.path.join(d, "d1")
        with open(os.path.join(d, "f2.rs"), "ab") as fh:
            fh.write(b"d" * 4096)
    # i. mtime-only touches (expected invisible to dir-stats)
    quiet = os.path.join(root, "src", "m00", "mod0000")  # a directory with no other change in this workload
    for i in range(5):
        p = os.path.join(quiet, f"{i:02d}.py") if os.path.isdir(quiet) else os.path.join(root, "wide", f"w{100 + i:06d}.json")
        os.utime(p, (1500000000 + int(time.time()) % 10000000,) * 2)
    ev["touched_mtime_only"] = 5
    ev["workload_ms"] = round((time.perf_counter() - a) * 1000, 1)
    return ev


def main():
    root, outside, out, label = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2]), sys.argv[3], sys.argv[4]
    os.makedirs(outside, exist_ok=True)
    tmp = os.path.dirname(os.path.abspath(out))
    before_tsv, after_tsv, pruned_tsv = [os.path.join(tmp, f"walk-{label}-{k}.tsv") for k in ("before", "after", "pruned")]
    ck = os.path.join(tmp, f"ckpt-{label}.tsv")
    log = {"label": label}
    cleanup(root, outside, label)
    # 1. checkpoint
    s, ms = walk(root, before_tsv); log["walk_before"] = {"summary": s, "ms": round(ms, 1)}
    before = load(before_tsv, root)
    origins = sorted(p for p, r in before.items() if r["kind"] == "d" and r["gencount"] > 0)
    with open_atomic(ck) as fh:
        for p in origins:
            fh.write(f"{before[p]['gencount']}\t{p}\n")
    a = time.perf_counter()
    ck_totals = {"": dsapi.get(root)}
    for p in origins:
        ck_totals[p] = dsapi.get(os.path.join(root, p))
    log["checkpoint"] = {"origins": len(origins), "totals_ms": round((time.perf_counter() - a) * 1000, 1), "root_gencount": ck_totals[""]["gen"]}
    # 2. workload
    held = {}
    log["workload"] = workload(root, outside, label, held)
    # 3. (ii) totals-only diff, immediately and after sync
    def totals_diff():
        a = time.perf_counter()
        changed = {}
        for p, t0 in ck_totals.items():
            try:
                t1 = dsapi.get(os.path.join(root, p) if p else root)
            except OSError as e:
                changed[p] = {"gone": e.errno, "dgen": None, "ddesc": -t0["desc"], "dphys": -t0["phys"]}
                continue
            if (t1["gen"], t1["desc"], t1["phys"]) != (t0["gen"], t0["desc"], t0["phys"]):
                changed[p] = {"dgen": t1["gen"] - t0["gen"], "ddesc": t1["desc"] - t0["desc"], "dphys": t1["phys"] - t0["phys"]}
        return changed, round((time.perf_counter() - a) * 1000, 1)
    ch_now, ms_now = totals_diff()
    os.sync()
    ch_sync, ms_sync = totals_diff()
    log["totals_only"] = {"immediate": {"changed_origins": len(ch_now), "ms": ms_now}, "after_sync": {"changed_origins": len(ch_sync), "ms": ms_sync, "changed": ch_sync}}
    # 4. (i) pruned walk
    s, ms = walk(root, pruned_tsv, ck); log["pruned_walk"] = {"summary": s, "ms": round(ms, 1)}
    visited = load(pruned_tsv, root)
    # 5. oracle
    s, ms = walk(root, after_tsv); log["walk_after"] = {"summary": s, "ms": round(ms, 1)}
    after = load(after_tsv, root)
    added = [p for p in after if p not in before]
    removed = [p for p in before if p not in after]
    size_changed, mtime_only = [], []
    for p in after:
        if p in before:
            b, x = before[p], after[p]
            if b["alloc"] != x["alloc"] or b["datalen"] != x["datalen"]:
                size_changed.append(p)
            elif b["mtime"] != x["mtime"] and x["kind"] != "d":
                mtime_only.append(p)
    def descended(d):
        return d == "" or (d in visited and visited[d]["kind"] == "d")

    def miss(p, removed_=False):
        if removed_:
            a = parent(p)
            while a and a not in after:
                a = parent(a)
            return not descended(a)
        return p not in visited
    misses = {"added": [p for p in added if miss(p)], "removed": [p for p in removed if miss(p, True)],
              "size_changed": [p for p in size_changed if miss(p)], "mtime_only": [p for p in mtime_only if miss(p)]}
    log["oracle"] = {"added": len(added), "removed": len(removed), "size_changed": len(size_changed), "mtime_only": len(mtime_only)}
    log["pruned_walk"].update({"visited": len(visited), "pruned_origins": sum(1 for r in visited.values() if r["kind"] == "P"),
                               "misses": {k: len(v) for k, v in misses.items()}, "miss_examples": {k: v[:5] for k, v in misses.items()}})
    # 6. attribution: per changed origin, oracle byte delta vs fsctl dphys (after sync)
    attr = {}
    for p, d in ch_sync.items():
        ob = subtree_alloc(before, p); oa = subtree_alloc(after, p)
        attr[p] = {"fsctl_dphys": d["dphys"], "oracle_dalloc": oa - ob, "match": d["dphys"] == oa - ob}
    unchanged_but_moved = [p for p in ck_totals if p not in ch_sync and subtree_alloc(before, p) != subtree_alloc(after, p)]
    log["attribution"] = {"origins_compared": len(attr), "mismatches": {p: v for p, v in attr.items() if not v["match"]},
                          "origins_with_oracle_delta_but_no_fsctl_change": unchanged_but_moved[:10], "n_silent": len(unchanged_but_moved)}
    # cleanup held handles
    os.close(held["log_fd"]); held["db"].close()
    write_text_atomic(out, json.dumps(log, indent=1, default=str))
    print(json.dumps({k: v for k, v in log.items() if k not in ("totals_only",)}, default=str)[:3000])
    print("totals_only:", json.dumps({"immediate": log["totals_only"]["immediate"], "after_sync_n": log["totals_only"]["after_sync"]["changed_origins"], "ms": log["totals_only"]["after_sync"]["ms"]}))


if __name__ == "__main__":
    main()
