#!/usr/bin/env python3
# Ported: REVIEW (the scratch working directory) comes from the environment instead of an absolute path.
"""Write-amplification benchmark: maintained (apfs.util -M) vs unmaintained tree, paired and interleaved.

  wa_bench.py ROOT_PARENT ROUNDS OUT.json

Creates ROOT_PARENT/wa-m (enabled with apfs.util -M while empty) and ROOT_PARENT/wa-u (plain).
Each round runs the same workload in both trees (order alternated): create 40 dirs x 50 files
(4 KiB each), append 4 KiB to every file, then delete half the files. Also a nested workload
at depth 12. Reports wall time per phase per tree. Run under timing-lock.
"""
import json
import os
import subprocess
import sys
import time
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402

APFS_UTIL = "/System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util"
FIXTURES = os.environ["REVIEW"] + "/catalog/fixtures/"


def workload(root, tag):
    t = {}
    base = os.path.join(root, tag)
    a = time.perf_counter()
    for d in range(40):
        dd = os.path.join(base, f"d{d:02d}")
        os.makedirs(dd)
        for f in range(50):
            with open(os.path.join(dd, f"f{f:03d}"), "w") as fh:
                fh.write("x" * 4096)
    t["create_2000"] = time.perf_counter() - a
    a = time.perf_counter()
    for d in range(40):
        dd = os.path.join(base, f"d{d:02d}")
        for f in range(50):
            with open(os.path.join(dd, f"f{f:03d}"), "a") as fh:
                fh.write("y" * 4096)
    t["append_2000"] = time.perf_counter() - a
    a = time.perf_counter()
    deep = base
    for i in range(12):
        deep = os.path.join(deep, f"l{i:02d}")
    os.makedirs(deep)
    for f in range(200):
        with open(os.path.join(deep, f"g{f:03d}"), "w") as fh:
            fh.write("z" * 4096)
    t["deep12_create_200"] = time.perf_counter() - a
    a = time.perf_counter()
    for d in range(40):
        dd = os.path.join(base, f"d{d:02d}")
        for f in range(0, 50, 2):
            os.unlink(os.path.join(dd, f"f{f:03d}"))
    t["delete_1000"] = time.perf_counter() - a
    a = time.perf_counter()
    os.sync()
    t["sync"] = time.perf_counter() - a
    return t


def main():
    parent, rounds, out = os.path.abspath(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    if not parent.startswith(FIXTURES):
        sys.exit("parent must be under the catalog fixtures directory")
    m, u = os.path.join(parent, "wa-m"), os.path.join(parent, "wa-u")
    if os.path.exists(m) or os.path.exists(u):
        sys.exit("refusing existing wa-m/wa-u")
    os.makedirs(m)
    os.makedirs(u)
    r = subprocess.run([APFS_UTIL, "-M", m], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit("enable failed: " + r.stderr)
    results = []
    for i in range(rounds):
        order = [("maintained", m), ("plain", u)] if i % 2 == 0 else [("plain", u), ("maintained", m)]
        row = {"round": i, "order": [o[0] for o in order]}
        for name, root in order:
            row[name] = workload(root, f"r{i:02d}")
        results.append(row)
        print(json.dumps(row))
    write_text_atomic(out, json.dumps(results, indent=1))
    # summary
    phases = list(results[0]["maintained"].keys())
    for ph in phases:
        mm = sorted(r["maintained"][ph] for r in results)
        uu = sorted(r["plain"][ph] for r in results)
        print(f"{ph:20s} maintained median={mm[len(mm)//2]*1000:8.1f} ms  min={mm[0]*1000:8.1f}   plain median={uu[len(uu)//2]*1000:8.1f} ms  min={uu[0]*1000:8.1f}   ratio(median)={mm[len(mm)//2]/uu[len(uu)//2]:.2f}")


if __name__ == "__main__":
    main()
