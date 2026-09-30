#!/usr/bin/env python3
"""Concurrent writers: 4 processes create/append/delete in one origin subtree for SECS
seconds, leaving some files open; then exactness of the fsctl totals vs an independent walk,
immediately and after sync(2).   concurrent.py ROOT OUT.json [SECS]"""
import json, os, re, subprocess, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402
DS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")
W = r'''
import os, sys, time, random
d, k, secs = sys.argv[1], int(sys.argv[2]), float(sys.argv[3])
me = os.path.join(d, "w%d" % k); os.makedirs(me, exist_ok=True)
r = random.Random(k); t_end = time.time() + secs; i = 0; held = []
while time.time() < t_end:
    p = os.path.join(me, "f%05d" % i)
    with open(p, "wb") as fh: fh.write(b"c" * r.choice([1024, 4096, 12288]))
    if i % 3 == 0:
        with open(os.path.join(me, "f%05d" % r.randint(0, i)), "ab") as fh: fh.write(b"a" * 2048)
    if i % 5 == 0 and i > 10:
        try: os.unlink(os.path.join(me, "f%05d" % r.randint(0, i - 1)))
        except FileNotFoundError: pass
    if i % 50 == 0:
        fd = os.open(os.path.join(me, "held%d" % (i // 50)), os.O_WRONLY | os.O_CREAT); os.write(fd, b"h" * 8192); held.append(fd)
    i += 1
print("writer", k, "ops", i, "held_open", len(held))
'''
def walk(p):
    r = subprocess.run([DS, "walk", p, "-o", "/dev/null"], capture_output=True, text=True).stdout
    m = re.search(r"entries=(\d+).*alloc=(\d+)", r); return int(m.group(1)), int(m.group(2))
def main():
    root, out = os.path.abspath(sys.argv[1]), sys.argv[2]; secs = sys.argv[3] if len(sys.argv) > 3 else "6"
    sub = os.path.join(root, "conc"); os.makedirs(sub, exist_ok=True)
    assert dsapi.apfs_util_M(sub)["rc"] == 0
    procs = [subprocess.Popen([sys.executable, "-c", W, sub, str(k), secs]) for k in range(4)]
    for p in procs: p.wait()
    res = {}
    for label in ("immediately_after_writers_exit", "after_sync"):
        if label == "after_sync": os.sync()
        g = dsapi.get(sub); e, a = walk(sub)
        res[label] = {"fsctl": (g["gen"], g["desc"], g["phys"]), "walk": (e, a), "desc_exact": g["desc"] == e, "phys_exact": g["phys"] == a}
    print(json.dumps(res, indent=1)); write_text_atomic(out, json.dumps(res, indent=1))
main()
