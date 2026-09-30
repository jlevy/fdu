#!/usr/bin/env python3
"""Persistence, safety and flag-removal tests on a disposable APFS disk image.

  dmg_tests.py IMAGE MOUNTPOINT OUT.json

IMAGE is a sparseimage created by the operator (hdiutil create -type SPARSE -fs APFS); it is
attached/detached repeatedly here with `-owners on -nobrowse`. Everything under MOUNTPOINT is
disposable. Steps:
  1. privilege: mark empty and populated dirs as the unprivileged user on the owned mount
  2. persistence: mark + populate, record, detach, attach, compare gencount/totals
  3. fsck_apfs -n (attach -nomount) before marking a populated tree and after
  4. forced detach in the middle of a write burst, re-attach, totals vs an independent walk
  5. flag removal attempts: fsctl 0xC1104A71 with other values at +4 and +0; rmdir; recreate
  6. moving a marked origin to the unmaintained volume root and back
"""
import json
import os
import re
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402

DS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")


def sh(*cmd, check=True, timeout=300):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    if check and r.returncode != 0:
        raise RuntimeError(f"{cmd}: rc={r.returncode} {r.stderr.strip()}")
    return r


def attach(image, nomount=False):
    args = ["hdiutil", "attach", image, "-nobrowse", "-owners", "on"]
    if nomount:
        args.append("-nomount")
    r = sh(*args, timeout=120)
    devs = re.findall(r"^(/dev/disk\d+s\d+)\s+41504653", r.stdout, re.M)  # APFS volume slice
    return devs[-1] if devs else None


def detach(target, force=False):
    args = ["hdiutil", "detach", target]
    if force:
        args.append("-force")
    for i in range(5):
        r = sh(*args, check=False, timeout=120)
        if r.returncode == 0:
            return r.stdout.strip() + r.stderr.strip()
        time.sleep(1)
    raise RuntimeError("detach failed: " + r.stderr)


def walk_totals(root):
    """Independent oracle: sum of ATTR_FILE_ALLOCSIZE over regular files (hard links once by fileid) and entry count."""
    r = sh(DS, "walk", root, "-o", "/dev/null")
    m = re.search(r"entries=(\d+) dirs=(\d+) files=(\d+) other=(\d+) pruned=\d+ wall_ms=([\d.]+) alloc=(\d+) data=(\d+)", r.stdout)
    return {"entries": int(m.group(1)), "dirs": int(m.group(2)), "files": int(m.group(3)), "alloc": int(m.group(6)), "data": int(m.group(7)), "wall_ms": float(m.group(5))}


def fsck(dev):
    r = sh("fsck_apfs", "-n", dev, check=False, timeout=600)
    return {"rc": r.returncode, "stdout": r.stdout, "stderr": r.stderr}


def populate(root, ndirs=20, nfiles=50, size=4096):
    for d in range(ndirs):
        dd = os.path.join(root, f"d{d:02d}")
        os.makedirs(dd, exist_ok=True)
        for f in range(nfiles):
            with open(os.path.join(dd, f"f{f:03d}"), "wb") as fh:
                fh.write(b"x" * size)


def state(path):
    try:
        g = dsapi.gen(path)["gencount"]
    except OSError as e:
        return {"error": e.errno}
    t = dsapi.get(path)
    return {"gencount": g, "fsctl": (t["gen"], t["desc"], t["phys"]), "fsctl_us": round(t["us"], 1)}


def main():
    image, mnt, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    log = {"uid": os.getuid()}
    if not os.path.ismount(mnt):
        attach(image)
    log["mount_line"] = [l for l in sh("mount").stdout.splitlines() if mnt in l]

    # 1. privilege on the owned mount
    e = os.path.join(mnt, "priv-empty"); os.makedirs(e)
    p = os.path.join(mnt, "priv-populated"); os.makedirs(p); populate(p, 5, 20)
    log["privilege"] = {"empty_M": dsapi.apfs_util_M(e), "empty_state": state(e),
                        "populated_M": dsapi.apfs_util_M(p), "populated_state": state(p), "populated_walk": walk_totals(p)}
    print("privilege:", json.dumps(log["privilege"]))

    # 2/3. fsck before marking a populated tree; mark; populate more; fsck after
    big = os.path.join(mnt, "persist"); os.makedirs(big); populate(big, 20, 50)
    sub = os.path.join(big, "d07")
    os.sync()
    detach(mnt)
    dev = attach(image, nomount=True)
    log["fsck_before_mark"] = fsck(dev)
    print("fsck before:", log["fsck_before_mark"]["rc"], log["fsck_before_mark"]["stdout"][-300:])
    detach(dev)
    attach(image)
    t0 = time.perf_counter()
    m1 = dsapi.apfs_util_M(big)
    t1 = time.perf_counter()
    m2 = dsapi.apfs_util_M(sub)
    populate(os.path.join(big, "d21"), 1, 30)
    with open(os.path.join(big, "d00", "f000"), "ab") as fh:
        fh.write(b"y" * 8192)
    os.unlink(os.path.join(big, "d01", "f001"))
    before = {"big": state(big), "sub": state(sub), "walk": walk_totals(big)}
    log["persistence"] = {"mark_big_ms": round((t1 - t0) * 1000, 1), "mark_rc": (m1["rc"], m2["rc"]), "before_detach": before}
    detach(mnt)
    dev = attach(image, nomount=True)
    log["fsck_after_mark"] = fsck(dev)
    print("fsck after mark:", log["fsck_after_mark"]["rc"], log["fsck_after_mark"]["stdout"][-300:])
    detach(dev)
    attach(image)
    after = {"big": state(big), "sub": state(sub), "walk": walk_totals(big)}
    log["persistence"]["after_reattach"] = after
    log["persistence"]["survived"] = before["big"] == after["big"] and before["sub"] == after["sub"]
    print("persistence:", json.dumps(log["persistence"]))

    # 4. forced detach mid-burst
    burst = os.path.join(mnt, "burst"); os.makedirs(burst)
    assert dsapi.apfs_util_M(burst)["rc"] == 0
    populate(burst, 10, 20)
    os.sync()
    pre = {"state": state(burst), "walk": walk_totals(burst)}
    writer = subprocess.Popen([sys.executable, "-c", f"""
import os, time
root = {burst!r}
i = 0
while True:
    d = os.path.join(root, 'b%03d' % (i % 50)); os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, 'f%06d' % i), 'wb') as fh: fh.write(b'z' * 8192)
    if i % 7 == 0:
        with open(os.path.join(root, 'd00', 'f000'), 'ab') as fh: fh.write(b'a' * 4096)
    if i % 11 == 0:
        try: os.unlink(os.path.join(root, 'd01', 'f%03d' % (i % 20)))
        except FileNotFoundError: pass
    i += 1
"""])
    time.sleep(2.0)
    t0 = time.perf_counter()
    r = sh("hdiutil", "detach", mnt, "-force", check=False, timeout=120)
    t1 = time.perf_counter()
    writer.kill(); writer.wait()
    log["forced_detach"] = {"detach_rc": r.returncode, "detach_ms": round((t1 - t0) * 1000), "detach_err": r.stderr.strip()[-300:], "pre": pre}
    dev = attach(image, nomount=True)
    log["forced_detach"]["fsck_after_force"] = fsck(dev)
    print("fsck after forced detach:", log["forced_detach"]["fsck_after_force"]["rc"], log["forced_detach"]["fsck_after_force"]["stdout"][-400:])
    detach(dev)
    attach(image)
    post = {"state": state(burst), "walk": walk_totals(burst), "persist_state": state(big), "persist_walk": walk_totals(big)}
    log["forced_detach"]["post"] = post
    # exactness: fsctl phys vs walk alloc, fsctl desc vs walk entries
    log["forced_detach"]["exact"] = (post["state"]["fsctl"][2] == post["walk"]["alloc"] and post["state"]["fsctl"][1] == post["walk"]["entries"])
    print("forced detach:", json.dumps({k: v for k, v in log["forced_detach"].items() if k != "fsck_after_force"}))

    # 5. flag removal attempts on a disposable origin
    fr = os.path.join(mnt, "flagtest"); os.makedirs(fr); populate(fr, 2, 10)
    assert dsapi.apfs_util_M(fr)["rc"] == 0
    attempts = []
    for w1, w0 in [(0, 0), (2, 0), (3, 0), (4, 0), (8, 0), (0x10, 0), (0x80000000, 0), (0xFFFFFFFF, 0), (0, 1), (0, 2), (1, 1), (0, 0x80000000)]:
        try:
            res = dsapi.mark(fr, w1, w0)
            outcome = {"ok": True, "gen": res["gen"], "desc": res["desc"], "phys": res["phys"], "words": {k: hex(v) for k, v in res["words"].items()}}
        except OSError as e:
            outcome = {"ok": False, "errno": e.errno, "strerror": e.strerror}
        st = state(fr)
        attempts.append({"w1": hex(w1), "w0": hex(w0), "call": outcome, "state_after": st})
        print("flag attempt", hex(w1), hex(w0), json.dumps(outcome), st)
    log["flag_removal_attempts"] = attempts
    # delete the origin, recreate same name
    shutil.rmtree(fr)
    os.makedirs(fr)
    log["rmtree_recreate"] = state(fr)
    # move an origin to the volume root sibling (unmaintained parent) and back
    mv = os.path.join(mnt, "priv-populated"); dst = os.path.join(mnt, "moved-origin")
    s0 = state(mv); os.rename(mv, dst); s1 = state(dst)
    with open(os.path.join(dst, "d00", "f000"), "ab") as fh:
        fh.write(b"m" * 4096)
    s2 = state(dst); os.rename(dst, mv); s3 = state(mv)
    log["move_origin"] = {"before": s0, "after_move": s1, "after_append_in_moved": s2, "after_move_back": s3}
    print("move origin:", json.dumps(log["move_origin"]))
    # cross-volume move (mv copies): destination loses the mark?
    # (kept inside the image: a rename across directories is not a copy, so instead test cp -R of an origin)
    cp = os.path.join(mnt, "copied-origin")
    sh("cp", "-R", mv, cp)
    log["cp_R_of_origin"] = state(cp)
    # 6. non-APFS error handling is covered elsewhere (tmpfs/devfs) — record fsctl on the mount root itself
    log["fsctl_on_volume_root"] = state(mnt)
    write_text_atomic(out_path, json.dumps(log, indent=1, default=str))
    print("wrote", out_path)


if __name__ == "__main__":
    main()
