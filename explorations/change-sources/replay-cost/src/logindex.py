"""Index a readable .fseventsd directory: per log file, compressed and raw bytes, record
count, id range, mtime, and coarse path-category record counts.

Read-only. Output: CSV (one row per file) plus a JSON summary with category tallies.
Record layout (3SLD, verified empirically on this host): page header 12 bytes
('3SLD', 4 unknown, u32 page length), records = path\\0 + u64 id + u32 flags + u64 node + 4
unknown (24-byte trailer).

usage: logindex.py FSEVENTSD_DIR OUT_CSV OUT_JSON [workers]
"""

from __future__ import annotations

import csv
import gzip
import json
import os
import struct
import sys
import time
from collections import Counter
from concurrent.futures import ProcessPoolExecutor

TRAILER = 24


def category(path: bytes) -> str:
    """Coarse, non-private category for a device-relative path."""
    parts = path.split(b"/")
    depth = len(parts)
    if b"target" in parts:
        return "cargo-target-like"
    if b"node_modules" in parts:
        return "node_modules"
    if b".git" in parts:
        return "git-internals"
    if b".fseventsd" in parts:
        return "fseventsd"
    if any(p.startswith(b".") for p in parts[:2]):
        return "dot-dirs-top2"
    if depth <= 2:
        return "shallow(depth<=2)"
    return "other"


def index_one(args: tuple[str, str]) -> dict:
    directory, name = args
    full = os.path.join(directory, name)
    gz = os.path.getsize(full)
    mtime = os.path.getmtime(full)
    raw = gzip.open(full, "rb").read()
    n = 0
    min_id = None
    max_id = 0
    cats: Counter[str] = Counter()
    pathbytes = 0
    off = 0
    pages = 0
    bad = 0
    while off + 12 <= len(raw):
        magic = raw[off : off + 4]
        if magic not in (b"1SLD", b"2SLD", b"3SLD"):
            bad += 1
            break
        plen = struct.unpack("<I", raw[off + 8 : off + 12])[0]
        if plen < 12:
            bad += 1
            break
        end = min(off + plen, len(raw))
        p = off + 12
        pages += 1
        while p < end:
            z = raw.find(b"\0", p, end)
            if z < 0 or z + 1 + TRAILER > end:
                bad += 1
                break
            path = raw[p:z]
            eid = struct.unpack_from("<Q", raw, z + 1)[0]
            p = z + 1 + TRAILER
            n += 1
            pathbytes += len(path)
            if min_id is None or eid < min_id:
                min_id = eid
            if eid > max_id:
                max_id = eid
            cats[category(path)] += 1
        off = end
    return {
        "name": name,
        "name_id": int(name, 16),
        "gz_bytes": gz,
        "raw_bytes": len(raw),
        "records": n,
        "pages": pages,
        "bad": bad,
        "min_id": min_id or 0,
        "max_id": max_id,
        "mtime": mtime,
        "path_bytes": pathbytes,
        "cats": dict(cats),
    }


def main() -> None:
    directory, out_csv, out_json = sys.argv[1:4]
    workers = int(sys.argv[4]) if len(sys.argv) > 4 else 4
    names = sorted(
        (n for n in os.listdir(directory) if len(n) >= 8 and all(c in "0123456789abcdef" for c in n)),
        key=lambda n: int(n, 16),
    )
    t0 = time.time()
    rows: list[dict] = []
    with ProcessPoolExecutor(max_workers=workers) as ex:
        for r in ex.map(index_one, [(directory, n) for n in names], chunksize=64):
            rows.append(r)
    elapsed = time.time() - t0
    cats: Counter[str] = Counter()
    for r in rows:
        cats.update(r["cats"])
    with open(out_csv, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["name", "name_id", "gz_bytes", "raw_bytes", "records", "pages", "bad", "min_id", "max_id", "mtime", "path_bytes"])
        for r in rows:
            w.writerow([r[k] for k in ("name", "name_id", "gz_bytes", "raw_bytes", "records", "pages", "bad", "min_id", "max_id", "mtime", "path_bytes")])
    summary = {
        "directory": directory,
        "files": len(rows),
        "gz_bytes": sum(r["gz_bytes"] for r in rows),
        "raw_bytes": sum(r["raw_bytes"] for r in rows),
        "records": sum(r["records"] for r in rows),
        "path_bytes": sum(r["path_bytes"] for r in rows),
        "bad_files": sum(1 for r in rows if r["bad"]),
        "oldest_mtime": min(r["mtime"] for r in rows),
        "newest_mtime": max(r["mtime"] for r in rows),
        "index_seconds": elapsed,
        "categories": dict(cats.most_common()),
        "name_is_upper_bound": sum(1 for r in rows if r["max_id"] <= r["name_id"]),
        "name_is_lower_bound": sum(1 for r in rows if r["min_id"] >= r["name_id"]),
    }
    with open(out_json, "w") as f:
        json.dump(summary, f, indent=1)
    print(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main()
