#!/usr/bin/env python3
"""Replicate the flag-removal observation with clean sequences on fresh directories of a
disposable volume. Never run this outside a disposable fixture.

  flag_unset.py MOUNTPOINT OUT.json

For each candidate value V at struct offset +4 (the word apfs.util -M sets to 1): build a
fresh populated tree (root + 3 subdirs x 40 files), mark root with apfs.util -M, record
state, call fsctl 0xC1104A71 with +4 = V once, record state, then append a file and record
state again (does anything still track?). State = gencount (getattrlist) and fsctl GET
(gen, desc, phys, call time). Also records the GET time on a 2,000-entry non-origin
subdirectory before/after the call (does an inherited directory keep a record?).
"""
import json
import os
import shutil
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402


def populate(root, ndirs, nfiles):
    for d in range(ndirs):
        dd = os.path.join(root, f"d{d:02d}")
        os.makedirs(dd, exist_ok=True)
        for f in range(nfiles):
            with open(os.path.join(dd, f"f{f:04d}"), "wb") as fh:
                fh.write(b"x" * 4096)


def state(path):
    g = dsapi.gen(path)["gencount"]
    ts = [dsapi.get(path) for _ in range(5)]
    t = ts[-1]
    return {"gencount": g, "fsctl": (t["gen"], t["desc"], t["phys"]), "fsctl_us_min": round(min(x["us"] for x in ts), 1)}


def main():
    mnt, out = sys.argv[1], sys.argv[2]
    base = os.path.join(mnt, "flag-unset")
    if os.path.exists(base):
        shutil.rmtree(base)
    os.makedirs(base)
    results = []
    for v in (2, 3, 0x10000, 0x100, 1):
        root = os.path.join(base, f"v{v:x}")
        os.makedirs(root)
        populate(root, 3, 40)
        big = os.path.join(root, "big"); os.makedirs(big); populate(big, 20, 100)  # 2,020 entries, non-origin
        r = dsapi.apfs_util_M(root)
        rec = {"value": hex(v), "mark_rc": r["rc"], "after_mark": state(root), "big_after_mark": state(big)}
        try:
            call = dsapi.mark(root, v, 0)
            rec["call"] = {"ok": True, "returned": (call["gen"], call["desc"], call["phys"]), "words": {k: hex(x) for k, x in call["words"].items()}}
        except OSError as e:
            rec["call"] = {"ok": False, "errno": e.errno}
        rec["after_call"] = state(root)
        rec["big_after_call"] = state(big)
        with open(os.path.join(root, "d00", "new.bin"), "wb") as fh:
            fh.write(b"y" * 8192)
        rec["after_append"] = state(root)
        # re-mark and see whether tracking resumes with exact totals
        r2 = dsapi.apfs_util_M(root)
        rec["remark_rc"] = r2["rc"]
        rec["after_remark"] = state(root)
        with open(os.path.join(root, "d01", "new2.bin"), "wb") as fh:
            fh.write(b"z" * 4096)
        rec["after_remark_append"] = state(root)
        results.append(rec)
        print(json.dumps(rec))
    # separately: an unmarked control tree of the same shape, GET latency (kernel walk) for scale
    ctrl = os.path.join(base, "control"); os.makedirs(ctrl); populate(ctrl, 3, 40)
    bigc = os.path.join(ctrl, "big"); os.makedirs(bigc); populate(bigc, 20, 100)
    results.append({"control_unmarked": {"root": state(ctrl), "big": state(bigc)}})
    print(json.dumps(results[-1]))
    with open(out, "w") as fh:
        json.dump(results, fh, indent=1)


if __name__ == "__main__":
    main()
