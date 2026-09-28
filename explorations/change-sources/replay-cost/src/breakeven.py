"""Break-even between history replay and a full metadata scan, from the measured model:

    replay_s = floor_s + scan_rate_s_per_MB * gz_MB_after_cursor + match_us * matching_records / 1e6

versus a warm full scan at walker_us_per_entry * N_entries / 1e6.

Inputs are taken from data/summary.json fits (scan floor and per-MB slope) and the
busy-root cells (per-matching-record cost) plus the hourly churn profile of the external
log (data/logindex.csv, by file mtime). Prints a table of break-even cursor ages for
several scope sizes and walker rates, for both the observed churn (MB/h) and a few
hypothetical churn rates.

usage: breakeven.py [SUMMARY_JSON] [LOGINDEX_CSV] [MATCH_US]
"""

from __future__ import annotations

import csv
import json
import sys
import time


def main() -> None:
    summary = json.load(open(sys.argv[1] if len(sys.argv) > 1 else "data/summary.json"))
    idx = sys.argv[2] if len(sys.argv) > 2 else "data/logindex.csv"
    match_us = float(sys.argv[3]) if len(sys.argv) > 3 else 10.0
    fit = summary["fits"]["gz_MB"]
    floor_s, per_mb = fit["intercept_s"], fit["slope_s_per_unit"]
    rows = [r for r in csv.DictReader(open(idx))]
    now = time.time()
    hourly = {}
    for r in rows:
        h = int((now - float(r["mtime"])) // 3600)
        hourly[h] = hourly.get(h, 0) + int(r["gz_bytes"]) / 1e6
    last24 = sum(hourly.get(h, 0) for h in range(24))
    print(f"model: replay_s = {floor_s:.3f} + {per_mb:.4f} * gz_MB  (+ {match_us:.0f} us per matching record); r2={fit['r2']}")
    print(f"observed external churn: {last24:.0f} MB compressed log in the last 24 h ({last24/24:.1f} MB/h mean, hourly max {max(hourly.get(h,0) for h in range(24)):.1f})")
    print()
    print("| scope entries N | walker us/entry | full scan s | replay budget = full scan: log MB | break-even age at 14 MB/h | at 5 MB/h | at 1 MB/h |")
    print("|---:|---:|---:|---:|---:|---:|---:|")
    for n in (20_000, 100_000, 450_000, 2_000_000, 8_600_000, 13_600_000):
        for us in (1.0, 3.0):
            full = n * us / 1e6
            mb = max(0.0, (full - floor_s) / per_mb)
            def age(rate):
                return f"{mb/rate:.2f} h" if mb > 0 else "never"
            print(f"| {n:,} | {us:.0f} | {full:.2f} | {mb:.1f} | {age(14)} | {age(5)} | {age(1)} |")
    print()
    print("matching-record term: a scope whose subtree produced M records since the cursor adds about", f"{match_us:.0f} us * M;")
    for m in (10_000, 100_000, 1_000_000, 3_000_000):
        print(f"  M={m:,}: +{m*match_us/1e6:.2f} s")


if __name__ == "__main__":
    main()
