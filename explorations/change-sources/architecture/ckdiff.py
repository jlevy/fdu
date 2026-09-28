#!/usr/bin/env python3
"""Diff two fdu JSON tree dumps by directory path: count changed directory roll-ups,
report magnitudes (no private paths in the shareable summary)."""
import json, sys, time
a, b, out = sys.argv[1], sys.argv[2], sys.argv[3]
def load(p):
    t0 = time.monotonic(); d = json.load(open(p)); t1 = time.monotonic()
    m = {}
    st = [d['reports'][0]['tree']]
    while st:
        n = st.pop()
        if n.get('kind') == 'dir':
            m[n['path']] = (n.get('allocated', 0), n.get('bytes', 0), n.get('files', 0), n.get('dirs', 0))
        for c in n.get('children') or []: st.append(c)
    return m, t1 - t0, time.monotonic() - t1
A, la, ia = load(a); B, lb, ib = load(b)
t0 = time.monotonic()
changed = []; added = 0; removed = 0
for p, va in A.items():
    vb = B.get(p)
    if vb is None: removed += 1; changed.append((p, -va[0], va[0], 0)); continue
    if va != vb: changed.append((p, vb[0] - va[0], va[0], vb[0]))
for p, vb in B.items():
    if p not in A: added += 1; changed.append((p, vb[0], 0, vb[0]))
changed.sort(key=lambda r: -abs(r[1]))
tdiff = time.monotonic() - t0
depth = lambda p: p.count('/')
with open(out, 'w') as f:
    f.write(f"dirs A={len(A)} B={len(B)} changed_rollups={len(changed)} (added {added}, removed {removed})\n")
    f.write(f"load A {la:.2f}s+{ia:.2f}s, load B {lb:.2f}s+{ib:.2f}s, diff {tdiff:.3f}s\n")
    f.write(f"root allocated delta: {B.get('', (0,))[0] - A.get('', (0,))[0]} bytes\n")
    f.write("top 15 |delta allocated| (depth, delta, before, after) -- paths withheld:\n")
    for p, d, bf, af in changed[:15]:
        f.write(f"  depth={depth(p)} delta={d:+d} before={bf} after={af}\n")
    hist = {}
    for p, d, bf, af in changed: hist[depth(p)] = hist.get(depth(p), 0) + 1
    f.write("changed dirs by depth: " + ", ".join(f"{k}:{v}" for k, v in sorted(hist.items())) + "\n")
print(open(out).read())
