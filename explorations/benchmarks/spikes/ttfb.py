"""Time to first stdout byte for the real CLI, paired and interleaved.

Not a harness job: the probe cannot see when bytes reach a terminal, and this is the one
signal fdu-n75m part 1 changes. Counterbalanced pairs, medians and a bootstrap interval
over paired differences, the same arithmetic the loop uses.
"""
import os, random, shutil, statistics, subprocess, sys, time, json
from pathlib import Path

root, control, candidate, trials, mode = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), sys.argv[5]
cache = {name: Path(f"/tmp/fdu-realtree/scratch/ttfb-cache-{name}") for name in ("control", "candidate")}
binaries = {"control": control, "candidate": candidate}

def run(name):
    env = dict(os.environ, XDG_CACHE_HOME=str(cache[name]), NO_COLOR="1")
    if mode == "first":
        shutil.rmtree(cache[name], ignore_errors=True)
    cache[name].mkdir(parents=True, exist_ok=True)
    start = time.perf_counter_ns()
    proc = subprocess.Popen([binaries[name], root], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env)
    first = None
    total = 0
    while True:
        chunk = os.read(proc.stdout.fileno(), 65536)
        if not chunk:
            break
        if first is None:
            first = time.perf_counter_ns() - start
        total += len(chunk)
    proc.wait()
    wall = time.perf_counter_ns() - start
    assert proc.returncode == 0, proc.returncode
    return first, wall, total

# Prepare: one untimed run each so the repeated-run mode starts with a snapshot present.
for name in binaries:
    shutil.rmtree(cache[name], ignore_errors=True)
    run(name)

samples = {"control": [], "candidate": []}
for i in range(trials):
    order = ("control", "candidate") if i % 2 == 0 else ("candidate", "control")
    for name in order:
        samples[name].append(run(name))

def med(xs): return statistics.median(xs)
ttfb = {n: [s[0] for s in samples[n]] for n in samples}
wall = {n: [s[1] for s in samples[n]] for n in samples}
diffs = [ (c - k) / k * 100 for c, k in zip(ttfb["candidate"], ttfb["control"]) ]
wdiffs = [ (c - k) / k * 100 for c, k in zip(wall["candidate"], wall["control"]) ]
rng = random.Random(7)
def boot(d):
    meds = []
    for _ in range(2000):
        meds.append(med([rng.choice(d) for _ in d]))
    meds.sort()
    return meds[int(0.025 * len(meds))], meds[int(0.975 * len(meds))]
print(json.dumps({
    "mode": mode, "trials": trials, "bytes": samples["control"][0][2],
    "ttfb_ms": {n: round(med(ttfb[n]) / 1e6, 1) for n in ttfb},
    "wall_ms": {n: round(med(wall[n]) / 1e6, 1) for n in wall},
    "ttfb_paired_change_pct": round(med(diffs), 2), "ttfb_ci95": [round(x, 2) for x in boot(diffs)],
    "wall_paired_change_pct": round(med(wdiffs), 2), "wall_ci95": [round(x, 2) for x in boot(wdiffs)],
    "ttfb_minus_wall_ms": {n: round((med(wall[n]) - med(ttfb[n])) / 1e6, 1) for n in ttfb},
}, indent=1))
