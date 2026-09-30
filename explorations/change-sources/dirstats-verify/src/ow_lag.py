#!/usr/bin/env python3
"""How long until an open writer's bytes reach the dir-stats physical total without fsync
or close (the periodic syncer).  ow_lag.py MARKED_DIR OUT.json [REPS]"""
import json, os, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402
def main():
    d, out = os.path.abspath(sys.argv[1]), sys.argv[2]; reps = int(sys.argv[3]) if len(sys.argv) > 3 else 3
    res = []
    for k in range(reps):
        p = os.path.join(d, f"owlag-{k}.log")
        p0 = dsapi.get(d)["phys"]
        fd = os.open(p, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
        t0 = time.perf_counter(); os.write(fd, b"L" * 262144); t_write = time.perf_counter() - t0
        g_first = dsapi.gencount(d)
        landed = None
        while time.perf_counter() - t0 < 90:
            time.sleep(0.25)
            if dsapi.get(d)["phys"] != p0:
                landed = round(time.perf_counter() - t0, 2); break
        p1 = dsapi.get(d)["phys"]
        os.close(fd)
        res.append({"rep": k, "write_ms": round(t_write * 1000, 2), "landed_after_s": landed, "phys_delta_when_landed": p1 - p0})
        print(res[-1])
        time.sleep(1)
    write_text_atomic(out, json.dumps(res, indent=1))
main()
