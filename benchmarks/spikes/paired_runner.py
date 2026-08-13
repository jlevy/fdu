#!/usr/bin/env python3
"""Adjacent-paired tool comparison, fdu-style: alternating order, wall via
monotonic clock around spawn+wait, rusage via os.wait4, paired bootstrap CI."""
import json, os, random, statistics, subprocess, sys, time

TREE = os.environ.get("SPIKE_TREE", "/tmp/fdu-spike/tree")
FDU = os.environ.get("SPIKE_FDU", "target/release/fdu")
HOME = os.path.expanduser("~")
DEVNULL = subprocess.DEVNULL

TOOLS = {
    "fdu-tree":    [FDU, "--cache", "off", TREE],
    "fdu-summary": [FDU, "--cache", "off", "--view", "summary", TREE],
    "fdu-warm":    [FDU, TREE],  # cache auto: verified warm open after seed
    "dut":         ["/home/user/dut-src/dut", TREE],
    "dust":        [HOME + "/.cargo/bin/dust", "-n", "10", TREE],
    "pdu":         [HOME + "/.cargo/bin/pdu", "--max-depth", "1", TREE],
    "diskus":      [HOME + "/.cargo/bin/diskus", TREE],
    "dua":         [HOME + "/.cargo/bin/dua", TREE],
    "fdu-mi-tree":     ["/home/user/bench/fdu-mimalloc", "--cache", "off", TREE],
    "fdu-mi-summary":  ["/home/user/bench/fdu-mimalloc", "--cache", "off", "--view", "summary", TREE],
    "spike-readdir":   ["/home/user/bench/walkspike", "readdir", TREE],
    "spike-getdents":  ["/home/user/bench/walkspike", "getdents", TREE],
    "spike-statx":     ["/home/user/bench/walkspike", "statx", TREE],
    "spike-narrow":    ["/home/user/bench/walkspike", "narrow", TREE],
    "spike-filesonly": ["/home/user/bench/walkspike", "filesonly", TREE],
    "spike-inosort":   ["/home/user/bench/walkspike", "inosort", TREE],
    "spike-uring":     ["/home/user/bench/walkspike", "uring", TREE],
}

def run_once(name):
    cmd = TOOLS[name]
    t0 = time.perf_counter_ns()
    proc = subprocess.Popen(cmd, stdout=DEVNULL, stderr=DEVNULL)
    _, status, ru = os.wait4(proc.pid, 0)
    wall = time.perf_counter_ns() - t0
    # Record the reaped status on the Popen so its destructor knows the child
    # is gone (otherwise every sample emits a ResourceWarning).
    proc.returncode = os.waitstatus_to_exitcode(status)
    if proc.returncode != 0:
        raise RuntimeError(f"{name} failed: {proc.returncode}")
    return {"wall_ns": wall, "utime": ru.ru_utime, "stime": ru.ru_stime,
            "maxrss_kb": ru.ru_maxrss, "minflt": ru.ru_minflt, "majflt": ru.ru_majflt}

def drop_caches():
    subprocess.run(["sync"], check=True)
    with open("/proc/sys/vm/drop_caches", "w") as f:
        f.write("3")
    time.sleep(0.15)

def paired(a, b, pairs, cold=False):
    rows = []
    for i in range(pairs):
        order = [a, b] if i % 2 == 0 else [b, a]
        sample = {}
        for name in order:
            if cold:
                drop_caches()
            sample[name] = run_once(name)
        rows.append(sample)
    return rows

def bootstrap_ci(diffs, n=4000):
    rng = random.Random(7)
    meds = []
    for _ in range(n):
        res = [diffs[rng.randrange(len(diffs))] for _ in diffs]
        meds.append(statistics.median(res))
    meds.sort()
    return meds[int(0.025 * n)], meds[int(0.975 * n)]

def summarize(tag, a, b, rows):
    diffs = [(r[b]["wall_ns"] - r[a]["wall_ns"]) / r[a]["wall_ns"] * 100 for r in rows]
    med = statistics.median(diffs)
    lo, hi = bootstrap_ci(diffs)
    med_a = statistics.median(r[a]["wall_ns"] for r in rows) / 1e6
    med_b = statistics.median(r[b]["wall_ns"] for r in rows) / 1e6
    cpu_a = statistics.median(r[a]["utime"] + r[a]["stime"] for r in rows)
    cpu_b = statistics.median(r[b]["utime"] + r[b]["stime"] for r in rows)
    sys_a = statistics.median(r[a]["stime"] for r in rows)
    sys_b = statistics.median(r[b]["stime"] for r in rows)
    rss_a = statistics.median(r[a]["maxrss_kb"] for r in rows) / 1024
    rss_b = statistics.median(r[b]["maxrss_kb"] for r in rows) / 1024
    out = {"tag": tag, "a": a, "b": b, "pairs": len(rows),
           "a_wall_ms": round(med_a, 1), "b_wall_ms": round(med_b, 1),
           "b_vs_a_pct": round(med, 2), "ci95": [round(lo, 2), round(hi, 2)],
           "a_cpu_s": round(cpu_a, 2), "b_cpu_s": round(cpu_b, 2),
           "a_sys_s": round(sys_a, 2), "b_sys_s": round(sys_b, 2),
           "a_rss_mb": round(rss_a, 1), "b_rss_mb": round(rss_b, 1)}
    print(json.dumps(out), flush=True)

def main():
    mode = sys.argv[1]
    matchups = [tuple(m.split(":")) for m in sys.argv[2].split(",")]
    pairs = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    cold = mode == "cold"
    if not cold:
        # Explicit warmups using only the tools the matchups reference: every
        # tool is itself a full-tree metadata pass, so two rounds warm the OS
        # caches and per-tool state without depending on binaries outside the
        # requested matchups.
        ordered = []
        for a, b in matchups:
            for t in (a, b):
                if t not in ordered:
                    ordered.append(t)
        for _ in range(2):
            for t in ordered:
                run_once(t)
    for a, b in matchups:
        rows = paired(a, b, pairs, cold=cold)
        summarize(mode, a, b, rows)

if __name__ == "__main__":
    main()
