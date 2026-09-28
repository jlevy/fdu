# Ported: the external st_dev comes from EXTERNAL_DEV, and the raw dev number is dropped from each result.
"""Produce commit-ready results from data/runs.jsonl: drop command lines, stderr/stdout
and any field that could carry a private path; keep labels, batch metadata, the
instrument's numeric record (paths appear only as lengths) and the log volume after each
run's cursor from the index. Also strips the directory path from logindex-summary.json.

usage: sanitize.py RUNS_JSONL LOGINDEX_CSV OUT_JSON
"""
from __future__ import annotations
import json, os, sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from since import load, since
KEEP_META = ("batch", "order", "label", "kind", "started_at", "finished_at", "wall_s", "rc", "loadavg", "cpu_before", "cpu_after", "event_lines", "series")
DROP_RESULT = ("label", "dev")
def main() -> None:
    rows = load(sys.argv[2])
    out = []
    for ln in open(sys.argv[1]):
        r = json.loads(ln)
        rec = {k: r[k] for k in KEEP_META if k in r}
        res = r.get("result")
        if isinstance(res, dict):
            rec["result"] = {k: v for k, v in res.items() if k not in DROP_RESULT}
            rec["volume"] = "external" if res.get("dev") == int(os.environ["EXTERNAL_DEV"]) else "internal"
            if rec["volume"] == "external" and res.get("cursor"):
                s = since(rows, int(res["cursor"]))
                rec["log_after_cursor"] = {k: s[k] for k in ("files_after", "gz_bytes_after", "raw_bytes_after", "records_after")}
        out.append(rec)
    json.dump({"schema": "fdu-replay-cost-review-v1", "host": "M1 Pro 10c/32GB, macOS 26.5.x, loaded", "date": "2026-09-27",
               "volumes": {"external": "APFS on USB SSD, noowners, ~13.6M inodes, log readable", "internal": "APFS Data volume, ~8.6M inodes, log root-only"},
               "runs": out}, open(sys.argv[3], "w"), indent=1)
    print(len(out), "runs written")
if __name__ == "__main__":
    main()
