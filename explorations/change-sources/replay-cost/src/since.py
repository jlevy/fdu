"""Given cursor event ids, report the external log volume (files, gz bytes, raw bytes,
records) strictly after each cursor, from data/logindex.csv.

A file whose max_id <= cursor holds nothing after the cursor. A file whose min_id > cursor
is entirely after it. A straddling file is counted fully in the 'files'/'bytes' columns
(fseventsd must decompress it whole to find the boundary) and its records are
apportioned linearly by id range for the 'records' column (approximation).

usage: since.py LOGINDEX_CSV CURSOR... | since.py LOGINDEX_CSV --jsonl RUNS_JSONL
"""

from __future__ import annotations

import csv
import json
import sys
import os
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import complete_lines  # noqa: E402


def load(path: str) -> list[dict]:
    rows = []
    with open(path) as f:
        for r in csv.DictReader(f):
            rows.append({k: (float(v) if k == "mtime" else int(v) if v.lstrip("-").isdigit() else v) for k, v in r.items()})
    rows.sort(key=lambda r: r["name_id"])
    return rows


def since(rows: list[dict], cursor: int) -> dict:
    files = gz = raw = recs = 0.0
    straddle = 0
    first_after_mtime = None
    for r in rows:
        if r["max_id"] <= cursor:
            continue
        files += 1
        gz += r["gz_bytes"]
        raw += r["raw_bytes"]
        if r["min_id"] > cursor:
            recs += r["records"]
        else:
            straddle += 1
            span = max(1, r["max_id"] - r["min_id"])
            recs += r["records"] * (r["max_id"] - cursor) / span
        if first_after_mtime is None:
            first_after_mtime = r["mtime"]
    return {
        "cursor": cursor,
        "files_after": int(files),
        "gz_bytes_after": int(gz),
        "raw_bytes_after": int(raw),
        "records_after": int(recs),
        "straddling_files": straddle,
        "first_file_after_mtime": first_after_mtime,
    }


def main() -> None:
    rows = load(sys.argv[1])
    if sys.argv[2] == "--jsonl":
        for line in complete_lines(sys.argv[3]):
            line = line.strip()
            if not line.startswith("{"):
                continue
            rec = json.loads(line)
            if "cursor" in rec:
                s = since(rows, int(rec["cursor"]))
                print(json.dumps({"label": rec.get("label"), **s}))
        return
    for c in sys.argv[2:]:
        print(json.dumps(since(rows, int(c))))


if __name__ == "__main__":
    main()
