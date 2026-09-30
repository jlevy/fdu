#!/usr/bin/env python3
"""Core dir-stats semantics on a fresh fixture, with in-process reads only (no subprocess
between a mutation and the read that observes it).

  semantics.py PARENT OUT.json [--replicates N] [--long-wait SECONDS] [--no-mark-check]

PARENT must be a directory the operator owns (external fixtures dir, or the internal temp dir
for the privilege test). Each replicate builds PARENT/sem-<k>/{root,outside}, marks root and
two children with apfs.util -M, runs the matrix, and leaves the fixture in place (the caller
deletes it). Every step records, for root and the origins A and B and the non-origin C:
gencount (getattrlist) and the fsctl totals (origins only), before and after, plus the
first in-process read after the operation returned (synchronous?) and its latency.
"""
import json
import mmap
import os
import shutil
import sqlite3
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402


def now_us():
    return time.perf_counter_ns() / 1000


SMALL = False


class Fixture:
    def __init__(self, base):
        self.root = os.path.join(base, "root")
        self.outside = os.path.join(base, "outside")
        self.A = os.path.join(self.root, "A")
        self.B = os.path.join(self.root, "B")
        self.C = os.path.join(self.root, "C")
        self.ops = os.path.join(self.root, "ops")
        self.deep = os.path.join(self.root, "deep", *[f"l{i:02d}" for i in range(12)])
        self.origins = {"root": self.root, "A": self.A, "B": self.B}
        self.watch = {"root": self.root, "A": self.A, "B": self.B, "C": self.C, "ops": self.ops}

    def build(self):
        os.makedirs(self.root)
        os.makedirs(self.outside)
        r = dsapi.apfs_util_M(self.root)
        assert r["rc"] == 0, r
        os.makedirs(self.B)
        r = dsapi.apfs_util_M(self.B)  # marked while empty
        assert r["rc"] == 0, r
        for d in (self.A, self.C, self.B):
            for s in range(3):
                sd = os.path.join(d, f"s{s}")
                os.makedirs(sd)
                for f in range(5 if SMALL else 20):
                    with open(os.path.join(sd, f"f{f:03d}.txt"), "w") as fh:
                        fh.write("payload " * 512)  # 4096 bytes
        os.makedirs(self.deep)
        with open(os.path.join(self.deep, "leaf.txt"), "w") as fh:
            fh.write("leaf " * 1024)
        os.makedirs(self.ops)
        for name in ["append", "openwriter", "truncate", "delete", "rename", "chmod", "utime", "xattr", "read", "clone-src",
                     "link-target", "move-out", "overwrite", "mmap", "hl-outside", "rsrc"]:
            with open(os.path.join(self.ops, name + ".txt"), "w") as fh:
                fh.write("payload " * 512)
        r = dsapi.apfs_util_M(self.A)  # marked AFTER population (the "populated directory" case)
        assert r["rc"] == 0, r
        os.sync()


def snap(fx):
    s = {}
    for k, p in fx.watch.items():
        try:
            s[k + ".gen"] = dsapi.gencount(p)
        except OSError as e:
            s[k + ".gen"] = f"ERR {e.errno}"
    for k, p in fx.origins.items():
        try:
            g = dsapi.get(p)
            s[k + ".fsctl"] = (g["gen"], g["desc"], g["phys"])
        except OSError as e:
            s[k + ".fsctl"] = f"ERR {e.errno}"
    return s


def run_matrix(fx, long_wait):
    log = []

    def step(name, fn, note="", first_read=("root",)):
        before = snap(fx)
        t0 = now_us()
        info = fn()
        t1 = now_us()
        # First in-process read after the operation returned: synchronous?
        first = {}
        for k in first_read:
            p = fx.watch[k]
            a = now_us()
            g = dsapi.gencount(p)
            b = now_us()
            first[k] = {"gen": g, "read_us": round(b - a, 1), "since_op_return_us": round(a - t1, 1), "changed": g != before[k + ".gen"]}
        after = snap(fx)
        delta = {}
        for k in set(before) | set(after):
            b, a = before.get(k), after.get(k)
            if isinstance(b, tuple) and isinstance(a, tuple):
                delta[k] = tuple(x - y for x, y in zip(a, b))
            elif isinstance(b, int) and isinstance(a, int):
                delta[k] = a - b
            else:
                delta[k] = (b, a)
        entry = {"step": name, "note": note, "op_us": round(t1 - t0, 1), "info": info if isinstance(info, dict) else {},
                 "first_read": first, "delta": delta, "before": before, "after": after}
        log.append(entry)
        d = {k: (f"{v:+d}" if isinstance(v, int) else str(v)) for k, v in delta.items()}
        print(f"{name:32s} root.gen {d['root.gen']:>4} A {d.get('A.gen','-'):>4} B {d.get('B.gen','-'):>4} C {d['C.gen']:>3} ops {d['ops.gen']:>3} | root fsctl d(gen,desc,phys)={d['root.fsctl']} A={d.get('A.fsctl','-')} B={d.get('B.fsctl','-')} | first={ {k: (v['changed'], v['since_op_return_us']) for k, v in first.items()} } {note}")
        return entry

    ops = fx.ops
    step("noop", lambda: None, "polling itself")
    step("sync", lambda: os.sync(), "sync(2)")

    # --- append + close
    def append_close():
        with open(os.path.join(ops, "append.txt"), "a") as fh:
            fh.write("more " * 2048)
    step("append_close", append_close, "append 10 KiB, close")

    # --- open writer: write() without fsync/close; is the read synchronous?
    held = {}
    def ow_open_write():
        fd = os.open(os.path.join(ops, "openwriter.txt"), os.O_WRONLY | os.O_APPEND)
        held["fd"] = fd
        n = os.write(fd, b"o" * 20480)
        return {"written": n, "fstat_size": os.fstat(fd).st_size}
    step("ow_write1_nosync", ow_open_write, "20 KiB write(), fd open, no fsync")
    step("ow_write2_nosync", lambda: {"written": os.write(held["fd"], b"p" * 20480)}, "second write(), same fd")
    step("ow_write3_same_page", lambda: {"written": os.pwrite(held["fd"], b"Q" * 10, 0)}, "pwrite 10 B at offset 0 (no size change)")
    if long_wait > 0:
        def ow_wait():
            g0 = dsapi.gencount(fx.root); p0 = dsapi.get(fx.root)["phys"]
            t0 = time.time(); ev = []
            while time.time() - t0 < long_wait:
                time.sleep(0.5)
                g = dsapi.gencount(fx.root); p = dsapi.get(fx.root)["phys"]
                if g != g0 or p != p0:
                    ev.append({"t_s": round(time.time() - t0, 1), "gen": g, "phys": p})
                    g0, p0 = g, p
                    if len(ev) >= 4:
                        break
            return {"events": ev, "waited_s": round(time.time() - t0, 1)}
        step("ow_wait_no_fsync", ow_wait, f"wait up to {long_wait}s: does phys catch up without fsync?")
    step("ow_fsync", lambda: os.fsync(held["fd"]), "fsync while open")
    step("ow_write4_after_fsync", lambda: {"written": os.write(held["fd"], b"r" * 4096)}, "write after fsync")
    step("ow_close", lambda: os.close(held["fd"]), "close")

    # --- sqlite WAL held open (Python stdlib)
    db = {}
    def sqlite_open_insert():
        c = sqlite3.connect(os.path.join(ops, "wal.db"))
        c.execute("pragma journal_mode=wal")
        c.execute("create table t(x blob)")
        c.execute("insert into t values (?)", (b"z" * (8192 if SMALL else 65536),))
        c.commit()
        db["c"] = c
        return {"files": sorted(f for f in os.listdir(ops) if f.startswith("wal.db"))}
    step("sqlite_wal_insert_open", sqlite_open_insert, "sqlite3 WAL: create+insert 64 KiB, connection held open")
    def sqlite_insert_more():
        db["c"].execute("insert into t values (?)", (b"y" * (8192 if SMALL else 65536),))
        db["c"].commit()
    step("sqlite_wal_insert2_open", sqlite_insert_more, "second 64 KiB insert, still open")
    step("sqlite_close", lambda: db["c"].close(), "close connection (checkpoint)")

    # --- in-place overwrite (no size change), truncate, unlink
    def overwrite():
        fd = os.open(os.path.join(ops, "overwrite.txt"), os.O_WRONLY)
        os.pwrite(fd, b"X" * 4096, 0)
        os.close(fd)
    step("overwrite_same_size", overwrite, "pwrite 4096 B over 4096 B file")
    step("truncate_0", lambda: os.truncate(os.path.join(ops, "truncate.txt"), 0), "truncate to 0")
    step("unlink", lambda: os.unlink(os.path.join(ops, "delete.txt")), "unlink 4 KiB file")

    # --- renames
    step("rename_within_ops", lambda: os.rename(os.path.join(ops, "rename.txt"), os.path.join(ops, "renamed.txt")), "rename within same dir (non-origin ops)")
    step("rename_ops_to_A", lambda: os.rename(os.path.join(ops, "renamed.txt"), os.path.join(fx.A, "renamed.txt")), "move file ops -> origin A")
    step("rename_A_to_B", lambda: os.rename(os.path.join(fx.A, "renamed.txt"), os.path.join(fx.B, "renamed.txt")), "move file origin A -> origin B")
    step("rename_dir_C_s0", lambda: os.rename(os.path.join(fx.C, "s0"), os.path.join(fx.C, "s0-renamed")), "rename populated non-origin dir in place")
    step("move_out", lambda: os.rename(os.path.join(ops, "move-out.txt"), os.path.join(fx.outside, "move-out.txt")), "move file out of tree")
    step("move_in", lambda: os.rename(os.path.join(fx.outside, "move-out.txt"), os.path.join(ops, "move-in.txt")), "move file back in")

    # --- non-changes
    step("chmod", lambda: os.chmod(os.path.join(ops, "chmod.txt"), 0o600), "chmod")
    step("utime", lambda: os.utime(os.path.join(ops, "utime.txt"), (1577836800, 1577836800)), "utime")
    step("xattr_set", lambda: subprocess.run(["xattr", "-w", "com.example.k", "v" * 100, os.path.join(ops, "xattr.txt")], check=True), "xattr 100 B")
    def read_file():
        with open(os.path.join(ops, "read.txt"), "rb") as fh:
            return {"n": len(fh.read())}
    step("read", read_file, "read file content")
    step("listdir", lambda: {"n": len(os.listdir(fx.A))}, "list A")

    # --- hard links
    step("hardlink_add_same_dir", lambda: os.link(os.path.join(ops, "link-target.txt"), os.path.join(ops, "link-alias.txt")), "link in same dir")
    step("hardlink_add_to_A", lambda: os.link(os.path.join(ops, "link-target.txt"), os.path.join(fx.A, "link-alias2.txt")), "link into origin A")
    step("hardlink_rm_alias", lambda: os.unlink(os.path.join(ops, "link-alias.txt")), "unlink alias in ops")
    step("hardlink_to_outside", lambda: os.link(os.path.join(ops, "hl-outside.txt"), os.path.join(fx.outside, "alias.txt")), "link inside file to outside")
    def write_via_outside():
        with open(os.path.join(fx.outside, "alias.txt"), "a") as fh:
            fh.write("outside " * 2048)
    step("write_via_outside_link", write_via_outside, "append 16 KiB through outside name")
    step("unlink_outside_alias", lambda: os.unlink(os.path.join(fx.outside, "alias.txt")), "remove outside name")

    # --- clone
    step("clone_cp_c", lambda: subprocess.run(["cp", "-c", os.path.join(ops, "clone-src.txt"), os.path.join(ops, "clone-dst.txt")], check=True), "cp -c 4 KiB")
    step("clone_from_outside", lambda: subprocess.run(["cp", "-c", os.path.join(ops, "clone-src.txt"), os.path.join(fx.outside, "clone-out.txt")], check=True), "cp -c inside -> outside")

    # --- mmap
    def mmap_store():
        fd = os.open(os.path.join(ops, "mmap.txt"), os.O_RDWR)
        mm = mmap.mmap(fd, 4096)
        mm[0:100] = b"M" * 100
        held["mm"] = mm; held["mfd"] = fd
    step("mmap_store", mmap_store, "mmap store, no msync")
    step("mmap_msync", lambda: held["mm"].flush(), "msync")
    def mmap_close():
        held["mm"].close(); os.close(held["mfd"])
    step("mmap_close", mmap_close, "munmap+close")

    # --- new directory inside the maintained tree: origin or inherited?
    def mkdir_new():
        p = os.path.join(fx.A, "newdir")
        os.makedirs(os.path.join(p, "inner"))
        with open(os.path.join(p, "inner", "f.txt"), "w") as fh:
            fh.write("n" * (10000 if SMALL else 100000))
        g = dsapi.gencount(p)
        t = dsapi.get(p)
        return {"newdir.gen": g, "newdir.fsctl": (t["gen"], t["desc"], t["phys"]), "newdir.fsctl_us": t["us"]}
    step("mkdir_in_A_plus_100k", mkdir_new, "new dir under A + 100 KB file; is newdir an origin?")

    # --- resource fork / xattr with data
    step("rsrc_fork_write", lambda: subprocess.run(["xattr", "-w", "com.apple.ResourceFork", "R" * 20000, os.path.join(ops, "rsrc.txt")], check=True), "20 KB resource fork")

    # --- sparse
    def sparse():
        with open(os.path.join(ops, "sparse.bin"), "wb") as fh:
            fh.seek((1 if SMALL else 50) * 1024 * 1024); fh.write(b"end")
    step("sparse_50M", sparse, "sparse file (50 MiB logical, 1 MiB in --small)")

    # --- symlink, rmdir
    step("symlink", lambda: os.symlink("append.txt", os.path.join(ops, "sym")), "symlink")
    step("rm_rf_C_s1", lambda: shutil.rmtree(os.path.join(fx.C, "s1")), "rm -rf C/s1 (20 files)")

    # --- origin operations
    def rename_B(to):
        os.rename(fx.B, to)
        fx.B = to; fx.origins["B"] = to; fx.watch["B"] = to
        g = dsapi.gencount(to); t = dsapi.get(to)
        return {"B.gen_now": g, "B.fsctl_now": (t["gen"], t["desc"], t["phys"]), "us": t["us"]}
    step("rename_origin_B", lambda: rename_B(fx.B + "-renamed"), "rename origin B within root")
    step("rename_origin_B_back", lambda: rename_B(fx.B[:-8]), "rename back")
    def move_origin_out():
        p = os.path.join(fx.outside, "A")
        os.rename(fx.A, p)
        fx.watch["A"] = p; fx.origins["A"] = p
        g = dsapi.gencount(p); t = dsapi.get(p)
        return {"A_outside.gen": g, "A_outside.fsctl": (t["gen"], t["desc"], t["phys"]), "us": t["us"]}
    step("move_origin_A_out", move_origin_out, "move origin A OUT of the tree", first_read=("root",))
    def append_in_moved_A():
        with open(os.path.join(fx.watch["A"], "s0", "f000.txt"), "a") as fh:
            fh.write("after-move " * 1000)
    step("append_in_A_outside", append_in_moved_A, "append inside moved-out origin A (still an origin?)")
    def move_origin_back():
        p = os.path.join(fx.root, "A")
        os.rename(fx.watch["A"], p)
        fx.watch["A"] = p; fx.origins["A"] = p; fx.A = p
    step("move_origin_A_back", move_origin_back, "move A back in")
    def mark_C():
        r = dsapi.apfs_util_M(fx.C)
        g = dsapi.gencount(fx.C); t = dsapi.get(fx.C)
        return {"rc": r["rc"], "C.gen": g, "C.fsctl": (t["gen"], t["desc"], t["phys"]), "us": t["us"]}
    step("mark_C_populated", mark_C, "apfs.util -M on populated non-origin C: does root gen move?")
    fx.origins["C"] = fx.C
    def append_in_C():
        with open(os.path.join(fx.C, "s2", "f000.txt"), "a") as fh:
            fh.write("c " * 2048)
    step("append_in_C", append_in_C, "append under new origin C")
    def rmtree_B():
        shutil.rmtree(fx.B)
        del fx.origins["B"]; del fx.watch["B"]
    step("rmtree_origin_B", rmtree_B, "rm -rf origin B")
    def mkdir_B_again():
        os.makedirs(fx.B)
        return {"B_new.gen": dsapi.gencount(fx.B)}
    step("mkdir_B_same_name", mkdir_B_again, "recreate B (same name): origin flag gone with the inode?")
    return log


def main():
    args = sys.argv[1:]
    reps = 3; long_wait = 0
    if "--replicates" in args:
        i = args.index("--replicates"); reps = int(args[i + 1]); del args[i:i + 2]
    if "--long-wait" in args:
        i = args.index("--long-wait"); long_wait = float(args[i + 1]); del args[i:i + 2]
    global SMALL
    if "--small" in args:
        args.remove("--small"); SMALL = True
    parent, out = os.path.abspath(args[0]), args[1]
    results = {"parent": parent, "mount": subprocess.run(["mount"], capture_output=True, text=True).stdout, "uid": os.getuid(), "replicates": []}
    for k in range(reps):
        base = os.path.join(parent, f"sem-{k}")
        if os.path.exists(base):
            shutil.rmtree(base)
        fx = Fixture(base)
        try:
            fx.build()
        except AssertionError as e:
            results["replicates"].append({"k": k, "build_error": str(e)})
            print("BUILD FAILED", e)
            break
        print(f"=== replicate {k} at {base}")
        results["replicates"].append({"k": k, "base": base, "log": run_matrix(fx, long_wait if k == 0 else 0)})
    write_text_atomic(out, json.dumps(results, indent=1, default=str))
    print("wrote", out)


if __name__ == "__main__":
    main()
