# Ported: the scratch top-level directory name comes from CHURN_SCRATCH_TOP; its class is labelled scratch-top-level.
"""Coarse churn categories for the external log (read-only): share of records by
top-level directory *class* (never printing private names), share under cargo-target-like
dirs, node_modules, .git, and the count of distinct depth-2 directories carrying most of
the churn. Also the share of records whose path is under a given device-relative prefix
(for the busy-root cell) and after a given cursor.

usage: churn.py FSEVENTSD_DIR CURSOR PREFIX OUT_JSON [workers] [PRIVATE_BUSIEST_DIR_OUT]
"""

from __future__ import annotations

import gzip
import json
import os
import struct
import sys
from collections import Counter
from concurrent.futures import ProcessPoolExecutor

TRAILER = 24
SCRATCH_TOP = os.environ["CHURN_SCRATCH_TOP"].encode()


def scan(args: tuple[str, str, int, bytes]) -> dict:
    directory, name, cursor, prefix = args
    raw = gzip.open(os.path.join(directory, name), "rb").read()
    top: Counter[str] = Counter()
    depth2: Counter[bytes] = Counter()
    n = after = under = target = nm = git = 0
    flags_hist: Counter[int] = Counter()
    off = 0
    while off + 12 <= len(raw):
        plen = struct.unpack("<I", raw[off + 8 : off + 12])[0]
        if plen < 12:
            break
        end = min(off + plen, len(raw))
        p = off + 12
        while p < end:
            z = raw.find(b"\0", p, end)
            if z < 0 or z + 1 + TRAILER > end:
                break
            path = raw[p:z]
            eid, fl = struct.unpack_from("<QI", raw, z + 1)
            p = z + 1 + TRAILER
            n += 1
            if eid <= cursor:
                continue
            after += 1
            parts = path.split(b"/")
            top[("scratch-top-level" if parts[0] == SCRATCH_TOP else "other-top-level")] += 1
            if len(parts) >= 2:
                depth2[b"/".join(parts[:2])] += 1
            if path.startswith(prefix):
                under += 1
            if b"target" in parts or any(x.startswith(b"target") for x in parts):
                target += 1
            if b"node_modules" in parts:
                nm += 1
            if b".git" in parts:
                git += 1
            flags_hist[fl & 0xFFFF] += 1
        off = end
    return {"n": n, "after": after, "under": under, "target": target, "nm": nm, "git": git,
            "top": dict(top), "depth2": {k.decode("utf8", "replace"): v for k, v in depth2.items()},
            "flags": dict(flags_hist)}


def main() -> None:
    directory, cursor, prefix, out = sys.argv[1], int(sys.argv[2]), sys.argv[3].encode(), sys.argv[4]
    workers = int(sys.argv[5]) if len(sys.argv) > 5 else 4
    names = sorted((n for n in os.listdir(directory) if len(n) >= 8 and all(c in "0123456789abcdef" for c in n)), key=lambda n: int(n, 16))
    tot = Counter()
    top: Counter[str] = Counter()
    depth2: Counter[str] = Counter()
    flags: Counter[int] = Counter()
    with ProcessPoolExecutor(max_workers=workers) as ex:
        for r in ex.map(scan, [(directory, n, cursor, prefix) for n in names], chunksize=64):
            for k in ("n", "after", "under", "target", "nm", "git"):
                tot[k] += r[k]
            top.update(r["top"])
            depth2.update(r["depth2"])
            flags.update({int(k): v for k, v in r["flags"].items()})
    after = tot["after"] or 1
    d2 = sorted(depth2.values(), reverse=True)
    cum = 0
    k90 = 0
    for v in d2:
        cum += v
        k90 += 1
        if cum >= 0.9 * after:
            break
    summary = {
        "cursor": cursor, "prefix_len": len(prefix), "records_total": tot["n"], "records_after_cursor": tot["after"],
        "share_under_prefix": round(tot["under"] / after, 4), "share_cargo_target_like": round(tot["target"] / after, 4),
        "share_node_modules": round(tot["nm"] / after, 4), "share_git_internals": round(tot["git"] / after, 4),
        "share_by_top_level_class": {k: round(v / after, 4) for k, v in top.items()},
        "distinct_depth2_dirs": len(depth2), "depth2_dirs_for_90pct": k90,
        "top5_depth2_shares": [round(v / after, 4) for v in d2[:5]],
        "flag_histogram_low16": {hex(k): v for k, v in flags.most_common(12)},
    }
    json.dump(summary, open(out, "w"), indent=1)
    print(json.dumps(summary, indent=1))
    # Private side file (gitignored attic, never reported): the busiest depth-2 directory,
    # used only as a replay filter for the project-scope cell.
    if depth2 and len(sys.argv) > 6:
        busiest = max(depth2.items(), key=lambda kv: kv[1])[0]
        with open(sys.argv[6], "w") as f:
            f.write(busiest + "\n")


if __name__ == "__main__":
    main()
