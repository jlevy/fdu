#!/usr/bin/env python3
"""Which name of a hard-linked inode does a dir-stats origin count? Cases on a fresh marked
fixture (PARENT/hl/root with origin B inside, PARENT/hl/outside):
  1. inside file, link to outside, unlink the inside name  -> does root's total drop?
  2. outside file, link into root, unlink the outside name -> does root's total rise?
  3. file in root (not in B), link into origin B, unlink the root name -> does B's total rise?
  4. file in origin B, link into root/other, unlink B's name -> does B's total drop?
  5. same inode linked twice inside root (two names) then remove the first name -> root total?
Each step prints (dgen, ddesc, dphys) for root and B.
  hardlink_primary.py PARENT OUT.json
"""
import json, os, shutil, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402

def mk(p, n=65536):
    with open(p, "wb") as fh: fh.write(b"h" * n)

def main():
    parent, out = os.path.abspath(sys.argv[1]), sys.argv[2]
    base = os.path.join(parent, "hl")
    if os.path.exists(base): shutil.rmtree(base)
    root, outside = os.path.join(base, "root"), os.path.join(base, "outside")
    B, other = os.path.join(root, "B"), os.path.join(root, "other")
    for d in (root, outside, B, other): os.makedirs(d)
    assert dsapi.apfs_util_M(root)["rc"] == 0 and dsapi.apfs_util_M(B)["rc"] == 0
    log = []
    def step(name, fn):
        r0, b0 = dsapi.get(root), dsapi.get(B); fn(); r1, b1 = dsapi.get(root), dsapi.get(B)
        d = {"step": name, "root": (r1["gen"]-r0["gen"], r1["desc"]-r0["desc"], r1["phys"]-r0["phys"]), "B": (b1["gen"]-b0["gen"], b1["desc"]-b0["desc"], b1["phys"]-b0["phys"])}
        log.append(d); print(f"{name:40s} root d={d['root']}  B d={d['B']}")
    # 1
    step("1a create root/f1", lambda: mk(os.path.join(root, "f1")))
    step("1b link f1 -> outside/f1", lambda: os.link(os.path.join(root, "f1"), os.path.join(outside, "f1")))
    step("1c unlink root/f1 (inode lives outside only)", lambda: os.unlink(os.path.join(root, "f1")))
    # 2
    step("2a create outside/f2", lambda: mk(os.path.join(outside, "f2")))
    step("2b link outside/f2 -> root/f2", lambda: os.link(os.path.join(outside, "f2"), os.path.join(root, "f2")))
    step("2c unlink outside/f2 (inode lives in root only)", lambda: os.unlink(os.path.join(outside, "f2")))
    step("2d append 16K to root/f2", lambda: open(os.path.join(root, "f2"), "ab").write(b"a" * 16384))
    # 3
    step("3a create root/f3", lambda: mk(os.path.join(root, "f3")))
    step("3b link root/f3 -> B/f3", lambda: os.link(os.path.join(root, "f3"), os.path.join(B, "f3")))
    step("3c unlink root/f3 (inode only in B now)", lambda: os.unlink(os.path.join(root, "f3")))
    step("3d append 16K to B/f3", lambda: open(os.path.join(B, "f3"), "ab").write(b"a" * 16384))
    # 4
    step("4a create B/f4", lambda: mk(os.path.join(B, "f4")))
    step("4b link B/f4 -> other/f4", lambda: os.link(os.path.join(B, "f4"), os.path.join(other, "f4")))
    step("4c unlink B/f4 (inode only in other now)", lambda: os.unlink(os.path.join(B, "f4")))
    # 5
    step("5a create root/f5", lambda: mk(os.path.join(root, "f5")))
    step("5b link root/f5 -> root/f5b", lambda: os.link(os.path.join(root, "f5"), os.path.join(root, "f5b")))
    step("5c unlink root/f5 (first name)", lambda: os.unlink(os.path.join(root, "f5")))
    step("5d unlink root/f5b (last name)", lambda: os.unlink(os.path.join(root, "f5b")))
    # oracle
    ds = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")
    import subprocess
    w = subprocess.run([ds, "walk", root, "-o", "/dev/null"], capture_output=True, text=True).stdout.strip()
    wb = subprocess.run([ds, "walk", B, "-o", "/dev/null"], capture_output=True, text=True).stdout.strip()
    fin = {"root_fsctl": dsapi.get(root), "B_fsctl": dsapi.get(B), "walk_root": w, "walk_B": wb}
    print(fin); log.append(fin)
    write_text_atomic(out, json.dumps(log, indent=1, default=str))
main()
