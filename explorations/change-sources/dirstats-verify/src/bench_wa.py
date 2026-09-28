#!/usr/bin/env python3
"""Paired, interleaved write-amplification benchmark at depth 12.

  bench_wa.py MARKED_ROOT CONTROL_ROOT LABEL ROUNDS OUT.json

Both roots must contain deep-0/d0/d0/... (11 levels). Each round creates a fresh work dir
under that leaf in each root (order alternates), then: create 500 x 4 KiB files, append
4 KiB to each, delete all. Reports per-phase medians, min, p90 and the paired ratio per
round. LABEL names the marking state of MARKED_ROOT (one-origin, depth3, all-levels).
Run under the timing lock.
"""
import json
import os
import statistics
import sys
import time


def leaf(root):
    return os.path.join(root, "deep-0", *(["d0"] * 11))


def work(d, n=500):
    t = {}
    os.makedirs(d)
    a = time.perf_counter()
    for f in range(n):
        with open(os.path.join(d, f"f{f:04d}"), "wb") as fh:
            fh.write(b"x" * 4096)
    t["create"] = time.perf_counter() - a
    a = time.perf_counter()
    for f in range(n):
        with open(os.path.join(d, f"f{f:04d}"), "ab") as fh:
            fh.write(b"y" * 4096)
    t["append"] = time.perf_counter() - a
    a = time.perf_counter()
    for f in range(n):
        os.unlink(os.path.join(d, f"f{f:04d}"))
    t["delete"] = time.perf_counter() - a
    os.rmdir(d)
    return t


def main():
    m, c, label, rounds, out = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
    rows = []
    for r in range(rounds):
        order = [("marked", m), ("control", c)] if r % 2 == 0 else [("control", c), ("marked", m)]
        row = {"round": r, "order": [o[0] for o in order]}
        for name, root in order:
            row[name] = work(os.path.join(leaf(root), f"wa-{label}-r{r:02d}"))
        rows.append(row)
    summary = {}
    for ph in ("create", "append", "delete"):
        mm = [x["marked"][ph] * 1000 for x in rows]
        cc = [x["control"][ph] * 1000 for x in rows]
        ratios = sorted(a / b for a, b in zip(mm, cc))
        summary[ph] = {"marked_ms": {"median": round(statistics.median(mm), 1), "min": round(min(mm), 1), "p90": round(sorted(mm)[int(len(mm) * 0.9)], 1)},
                       "control_ms": {"median": round(statistics.median(cc), 1), "min": round(min(cc), 1), "p90": round(sorted(cc)[int(len(cc) * 0.9)], 1)},
                       "paired_ratio": {"median": round(statistics.median(ratios), 3), "min": round(ratios[0], 3), "max": round(ratios[-1], 3)}}
        print(f"{label:12s} {ph:7s} marked med {summary[ph]['marked_ms']['median']:7.1f} ms  control med {summary[ph]['control_ms']['median']:7.1f} ms  paired ratio med {summary[ph]['paired_ratio']['median']:.3f} [{ratios[0]:.3f}..{ratios[-1]:.3f}]")
    with open(out, "w") as fh:
        json.dump({"label": label, "rounds": rows, "summary": summary}, fh, indent=1)


if __name__ == "__main__":
    main()
