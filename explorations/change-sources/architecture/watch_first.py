#!/usr/bin/env python3
# Ported: the fdu binary comes from FDU (default: fdu on PATH) instead of an absolute path.
"""Time `fdu --watch` to its first summary record: snapshot load + full reconcile.

Runs fdu with `--cache read-only` (loads the snapshot, verifies every entry, writes
nothing), waits for the first `"view": "summary"` JSON Lines record, samples the
process RSS, then terminates it. Prints one TSV line.
Usage: watch_first.py LABEL ROOT CACHE_DIR
"""
import subprocess, sys, time, os, signal

label, root, cache = sys.argv[1:4]
cmd = [os.environ.get("FDU", "fdu"), root, "--watch", "--view", "summary",
       "--cache", "read-only", "--cache-dir", cache, "--format", "jsonl",
       "--color", "never", "--progress", "never"]
t0 = time.monotonic()
p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
first = None; src = None; summ = None
try:
    for line in p.stdout:
        if '"provenance"' in line and src is None:
            i = line.find('"source": "'); src = line[i+11:line.find('"', i+11)] if i >= 0 else "?"
        if '"view": "summary"' in line:
            first = time.monotonic() - t0
            summ = line.strip()[:160]
            break
finally:
    rss = subprocess.run(["ps", "-o", "rss=", "-p", str(p.pid)], capture_output=True, text=True).stdout.strip()
    p.send_signal(signal.SIGTERM)
    try:
        p.wait(timeout=10)
    except subprocess.TimeoutExpired:
        p.kill()
err = p.stderr.read()[:300].replace("\n", " | ")
print(f"{label}\t{root}\t{first if first is None else round(first,3)}\t{src}\t{rss}\t{summ}\t{err}")
