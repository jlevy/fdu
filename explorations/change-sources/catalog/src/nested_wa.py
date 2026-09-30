#!/usr/bin/env python3
# Ported: REVIEW (the scratch working directory) comes from the environment instead of an absolute path.
"""Nested-origin write amplification: 12 origins along a chain vs a plain chain. Paired, alternated."""
import os, subprocess, sys, time, json
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402
U="/System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util"
base=sys.argv[1]; rounds=int(sys.argv[2])
assert base.startswith(os.environ["REVIEW"] + "/catalog/fixtures/")
def chain(root):
    d=root
    for i in range(12): d=os.path.join(d,f"n{i:02d}")
    os.makedirs(d); return d
m=chain(os.path.join(base,"nest-m")); p=chain(os.path.join(base,"nest-p"))
# enable an origin at every level of the maintained chain (13 origins)
d=os.path.join(base,"nest-m"); subprocess.run([U,"-M",d],check=True,capture_output=True)
for i in range(12):
    d=os.path.join(d,f"n{i:02d}"); subprocess.run([U,"-M",d],check=True,capture_output=True)
def work(leaf,tag):
    t={}
    a=time.perf_counter()
    for f in range(500):
        with open(os.path.join(leaf,f"{tag}-{f}"),"w") as fh: fh.write("x"*4096)
    t["create_500"]=time.perf_counter()-a; a=time.perf_counter()
    for f in range(500):
        with open(os.path.join(leaf,f"{tag}-{f}"),"a") as fh: fh.write("y"*4096)
    t["append_500"]=time.perf_counter()-a; a=time.perf_counter()
    for f in range(500): os.unlink(os.path.join(leaf,f"{tag}-{f}"))
    t["delete_500"]=time.perf_counter()-a
    return t
res=[]
for r in range(rounds):
    order=[("m",m),("p",p)] if r%2==0 else [("p",p),("m",m)]
    row={"round":r}
    for k,leaf in order: row[k]=work(leaf,f"r{r}")
    res.append(row)
for ph in ("create_500","append_500","delete_500"):
    mm=sorted(x["m"][ph] for x in res); pp=sorted(x["p"][ph] for x in res)
    print(f"{ph:12s} 13-origins median={mm[len(mm)//2]*1000:7.1f} ms min={mm[0]*1000:7.1f}  plain median={pp[len(pp)//2]*1000:7.1f} ms min={pp[0]*1000:7.1f}  ratio={mm[len(mm)//2]/pp[len(pp)//2]:.2f}")
write_text_atomic(sys.argv[3], json.dumps(res,indent=1))
