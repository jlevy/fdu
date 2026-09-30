# Ported: the external volume's st_dev comes from EXTERNAL_DEV instead of a host-specific constant.
"""Summarize data/runs.jsonl against data/logindex.csv.

Produces data/summary.json and prints Markdown tables: per-cell statistics (n, min,
median, max of history_ms, fseventsd CPU delta, events) and, for the external quiet-filter
age sweep, the log volume after each run's cursor with derived rates (MB/s, records/s,
files/s) and simple least-squares fits of history_ms against gz bytes, files, records
and age.

usage: analyze.py [RUNS_JSONL] [LOGINDEX_CSV] [OUT_JSON]
"""

from __future__ import annotations

import json
import os
import statistics as st
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from since import load, since  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import complete_lines, write_text_atomic  # noqa: E402

EXT_DEV = int(os.environ["EXTERNAL_DEV"])


def fit(xs: list[float], ys: list[float]) -> tuple[float, float, float]:
    """least squares y = a + b x ; returns (a, b, r2)"""
    n = len(xs)
    if n < 2:
        return (float("nan"),) * 3
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    b = sxy / sxx if sxx else float("nan")
    a = my - b * mx
    ss_res = sum((y - (a + b * x)) ** 2 for x, y in zip(xs, ys))
    ss_tot = sum((y - my) ** 2 for y in ys)
    return a, b, 1 - ss_res / ss_tot if ss_tot else float("nan")


def main() -> None:
    runs_path = sys.argv[1] if len(sys.argv) > 1 else "data/runs.jsonl"
    idx_path = sys.argv[2] if len(sys.argv) > 2 else "data/logindex.csv"
    out_path = sys.argv[3] if len(sys.argv) > 3 else "data/summary.json"
    rows = load(idx_path)
    runs = [json.loads(ln) for ln in complete_lines(runs_path) if ln.strip()]
    recs = [r for r in runs if r.get("kind") in ("run", "bg-end") and r.get("result")]

    # 1. per-cell (label with trailing -rN / -N stripped) stats
    groups: dict[str, list[dict]] = defaultdict(list)
    for r in recs:
        base = r["label"]
        for suffix in ("-r1", "-r2", "-r3", "-1", "-2", "-3"):
            if base.endswith(suffix):
                base = base[: -len(suffix)]
                break
        groups[base].append(r)
    cell_table = []
    for base, rs in groups.items():
        h = [x["result"]["history_ms"] for x in rs]
        c = [x["result"]["fseventsd_cpu_delta"] for x in rs]
        e = [x["result"]["events"] for x in rs]
        done = all(x["result"]["history_done"] for x in rs)
        cell_table.append({
            "cell": base, "n": len(rs), "batch": rs[0]["batch"], "dev": rs[0]["result"]["dev"],
            "age_s": rs[0]["result"]["age_s"], "flags": rs[0]["result"]["flags"],
            "history_ms_min": min(h), "history_ms_med": st.median(h), "history_ms_max": max(h),
            "cpu_delta_med": st.median(c), "cpu_share_med": st.median([ci / (hi / 1e3) if hi > 0 else 0 for ci, hi in zip(c, h)]),
            "events_med": st.median(e), "all_done": done,
            "first_cb_ms_med": st.median([x["result"]["first_cb_ms"] for x in rs]),
        })
    cell_table.sort(key=lambda x: (x["batch"], x["cell"]))

    # 2. external quiet-filter sweep: volume after cursor and rates
    sweep = []
    for r in recs:
        res = r["result"]
        if res["dev"] != EXT_DEV or res["nfilters"] != 1 or res["events"] > 3 or not res["history_done"]:
            continue
        if not (r["batch"].startswith("sweep") or r["batch"] in ("bytes-vs-age", "flags", "load") or r["label"].startswith(("filt-quiet", "flush", "rep-", "noload", "bl-base", "bl-after", "bl-probe", "multi-seq", "conc-same-bg", "conc-cross-ext", "conc-cross2-ext", "ctl-quiet"))):
            continue
        s = since(rows, int(res["cursor"]))
        hs = res["history_ms"] / 1e3
        sweep.append({
            "label": r["label"], "batch": r["batch"], "age_s": res["age_s"], "flags": res["flags"],
            "history_s": round(hs, 3), "cpu_s": res["fseventsd_cpu_delta"],
            "gz_MB": round(s["gz_bytes_after"] / 1e6, 1), "raw_MB": round(s["raw_bytes_after"] / 1e6, 1),
            "files": s["files_after"], "records_M": round(s["records_after"] / 1e6, 2),
            "gz_MBps": round(s["gz_bytes_after"] / 1e6 / hs, 2) if hs > 0 else None,
            "raw_MBps": round(s["raw_bytes_after"] / 1e6 / hs, 1) if hs > 0 else None,
            "files_per_s": round(s["files_after"] / hs, 0) if hs > 0 else None,
            "records_per_s": round(s["records_after"] / hs, 0) if hs > 0 else None,
            "cpu_share": round(res["fseventsd_cpu_delta"] / hs, 2) if hs > 0 else None,
            "loadavg1": round(r["loadavg"][0], 1) if r.get("loadavg") else None,
        })
    sweep.sort(key=lambda x: (x["age_s"], x["label"]))
    clean = [x for x in sweep if x["batch"].startswith("sweep") or x["batch"] == "bytes-vs-age" or x["label"].startswith(("filt-quiet", "rep-", "noload", "flags16"))]
    fits = {}
    if len(clean) >= 3:
        ys = [x["history_s"] for x in clean]
        for key, name in (("gz_MB", "gz_MB"), ("raw_MB", "raw_MB"), ("files", "files"), ("records_M", "records_M"), ("age_s", "age_h")):
            xs = [x[key] / (3600 if key == "age_s" else 1) for x in clean]
            a, b, r2 = fit(xs, ys)
            fits[name] = {"intercept_s": round(a, 3), "slope_s_per_unit": round(b, 5), "r2": round(r2, 4), "n": len(clean)}
        cs = [x["cpu_s"] for x in clean]
        a, b, r2 = fit([x["gz_MB"] for x in clean], cs)
        fits["cpu_vs_gz_MB"] = {"intercept_s": round(a, 3), "slope_s_per_MB": round(b, 5), "r2": round(r2, 4)}

    summary = {"cells": cell_table, "ext_quiet_sweep": sweep, "fits": fits, "n_runs": len(recs)}
    write_text_atomic(out_path, json.dumps(summary, indent=1))

    print("## Cells\n")
    print("| cell | batch | n | dev | age | flags | hist min/med/max (s) | cpu med (s) | cpu share | events med | first cb (ms) | all done |")
    print("|---|---|---:|---|---:|---:|---|---:|---:|---:|---:|---|")
    for c in cell_table:
        print(f"| {c['cell']} | {c['batch']} | {c['n']} | {'ext' if c['dev']==EXT_DEV else 'int'} | {c['age_s']} | {c['flags']} | "
              f"{c['history_ms_min']/1e3:.2f} / {c['history_ms_med']/1e3:.2f} / {c['history_ms_max']/1e3:.2f} | {c['cpu_delta_med']:.2f} | "
              f"{c['cpu_share_med']:.2f} | {c['events_med']:.0f} | {c['first_cb_ms_med']:.1f} | {c['all_done']} |")
    print("\n## External quiet-filter sweep with log volume after cursor\n")
    print("| label | age | hist s | cpu s | share | gz MB | raw MB | files | recs M | gz MB/s | raw MB/s | files/s | recs/s | load1 |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for x in sweep:
        print(f"| {x['label']} | {x['age_s']} | {x['history_s']:.2f} | {x['cpu_s']:.2f} | {x['cpu_share']} | {x['gz_MB']} | {x['raw_MB']} | {x['files']} | {x['records_M']} | {x['gz_MBps']} | {x['raw_MBps']} | {x['files_per_s']} | {x['records_per_s']} | {x['loadavg1']} |")
    print("\n## Fits (history_s vs predictor, external quiet sweep)\n")
    print(json.dumps(fits, indent=1))


if __name__ == "__main__":
    main()
