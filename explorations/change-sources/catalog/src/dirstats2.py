#!/usr/bin/env python3
# Ported: REVIEW (the scratch working directory) comes from the environment instead of an absolute path.
"""Second dir-stats experiment: origin rename, many nested origins (pruned walk), outside
hard-link write, mmap write, and gencount localization. Uses an existing maintained fixture.

  dirstats2.py ROOT OUT.json
"""
import json
import mmap
import os
import re
import subprocess
import sys
import time

APFS_UTIL = "/System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util"
REVIEW = os.environ["REVIEW"]
PROBE = REVIEW + "/catalog/attic/extattr_walk"
FSCTL = REVIEW + "/catalog/attic/dirstat_fsctl"
FIXTURES = REVIEW + "/catalog/fixtures/"


def stats(path):
    r = subprocess.run([FSCTL, path], capture_output=True, text=True)
    m = re.search(r"gen_count=(\d+) descendants=(\d+) physical_size=(\d+)", r.stdout)
    return {"gen": int(m.group(1)), "desc": int(m.group(2)), "phys": int(m.group(3))} if m else {"error": r.stdout.strip()}


def gencount_map(root):
    """Walk with getattrlistbulk and return {relpath: recursive_gencount} for directories with nonzero gencount."""
    lst = os.path.join(REVIEW, "catalog", "attic", "gc-list.tsv")
    subprocess.run([PROBE, root, "--mode", "ext", "--gencount-list", lst], capture_output=True, text=True, check=True)
    out = {}
    with open(lst) as fh:
        for line in fh:
            gc, depth, path = line.rstrip("\n").split("\t")
            out[path] = int(gc)
    return out


def main():
    root, out_path = os.path.abspath(sys.argv[1]), sys.argv[2]
    if not root.startswith(FIXTURES):
        sys.exit("root must be under fixtures")
    log = {}

    # 1. Rename the origin directory itself.
    moved = root + "-moved"
    before = stats(root)
    os.rename(root, moved)
    after = stats(moved)
    os.rename(moved, root)
    log["origin_rename"] = {"before": before, "after_rename": after, "after_rename_back": stats(root)}
    print("origin rename:", log["origin_rename"])

    # 2. Enable origins on every d??/s? directory (50) and on every d?? (10) -> 61 origins incl root.
    subdirs = sorted(d for d in os.listdir(root) if re.match(r"d\d\d", d))
    enabled = []
    t0 = time.perf_counter()
    for d in subdirs:
        p = os.path.join(root, d)
        r = subprocess.run([APFS_UTIL, "-M", p], capture_output=True, text=True)
        enabled.append((d, r.returncode))
        for s in sorted(os.listdir(p)):
            sp = os.path.join(p, s)
            if os.path.isdir(sp):
                r = subprocess.run([APFS_UTIL, "-M", sp], capture_output=True, text=True)
                enabled.append((f"{d}/{s}", r.returncode))
    enable_ms = (time.perf_counter() - t0) * 1000
    gm0 = gencount_map(root)
    log["many_origins"] = {"enabled": len(enabled), "failed": [e for e in enabled if e[1] != 0], "enable_ms_total": round(enable_ms, 1),
                           "dirs_with_nonzero_gencount_after_enable": len(gm0)}
    print("many origins:", log["many_origins"])

    # 3. One leaf append: which origins' gencounts change? (pruned-walk demonstration)
    target = os.path.join(root, subdirs[4], "s2", "file-007.txt")
    root_before = stats(root)
    with open(target, "a") as fh:
        fh.write("localize " * 512)
    gm1 = gencount_map(root)
    changed = sorted(p for p in set(gm0) | set(gm1) if gm0.get(p) != gm1.get(p))
    log["localization"] = {"target": os.path.relpath(target, root), "changed_gencounts": {p: (gm0.get(p), gm1.get(p)) for p in changed},
                           "root_before": root_before, "root_after": stats(root), "origin_stats_after": stats(os.path.join(root, subdirs[4], "s2"))}
    print("localization:", json.dumps(log["localization"]))

    # 4. Hard link to outside the tree, then write through the outside path.
    inside = os.path.join(root, "ops", "append.txt")
    outside = os.path.join(os.path.dirname(root), "outside-alias.txt")
    if os.path.exists(outside):
        os.unlink(outside)
    b = stats(root)
    os.link(inside, outside)
    a1 = stats(root)
    with open(outside, "a") as fh:
        fh.write("outside " * 2048)
    a2 = stats(root)
    log["outside_hardlink_write"] = {"before": b, "after_link": a1, "after_write_via_outside_path": a2,
                                     "inside_size_now": os.lstat(inside).st_size}
    print("outside hardlink write:", log["outside_hardlink_write"])
    os.unlink(outside)

    # 5. mmap write (no write(2)); then msync; then close.
    p = os.path.join(root, "ops", "mmap.bin")
    with open(p, "wb") as fh:
        fh.write(b"\0" * 65536)
    b = stats(root)
    fd = os.open(p, os.O_RDWR)
    mm = mmap.mmap(fd, 65536)
    mm[0:8192] = b"M" * 8192
    a_dirty = stats(root)
    mm.flush()
    a_msync = stats(root)
    mm.close()
    os.close(fd)
    a_close = stats(root)
    os.sync()
    a_sync = stats(root)
    log["mmap_write"] = {"before": b, "after_store": a_dirty, "after_msync": a_msync, "after_close": a_close, "after_sync": a_sync}
    print("mmap write:", log["mmap_write"])

    # 6. Bulk-read cost: getattrlistbulk with gencount over the tree vs plain (timing, small tree, informal).
    ts = []
    for mode in ("base", "ext", "base", "ext"):
        r = subprocess.run([PROBE, root, "--mode", mode], capture_output=True, text=True)
        m = re.search(r"entries=(\d+).*wall_ms=([\d.]+)", r.stdout)
        ts.append((mode, int(m.group(1)), float(m.group(2))))
    log["bulk_read_timing_informal"] = ts
    print("bulk timing:", ts)

    with open(out_path, "w") as fh:
        json.dump(log, fh, indent=1)


if __name__ == "__main__":
    main()
