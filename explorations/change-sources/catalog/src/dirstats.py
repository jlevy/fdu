#!/usr/bin/env python3
# Ported: REVIEW (the scratch working directory) comes from the environment instead of an absolute path.
"""APFS maintain-dir-stats experiment on an owned fixture (external volume only).

  dirstats.py populate ROOT           create the tree under ROOT (ROOT must already be enabled with apfs.util -M)
  dirstats.py matrix ROOT OUT.json    run the operation matrix, recording gen-count / totals after every step
  dirstats.py stats DIR               print apfs.util -S and getattrlist gencount for DIR

Every step records: apfs.util -S (descendants, physical size, gen-count) of ROOT and of the
touched subdirectory, plus ATTR_CMNEXT_RECURSIVE_GENCOUNT via the C probe.
"""
import json
import os
import re
import subprocess
import sys
import time

APFS_UTIL = "/System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util"
REVIEW = os.environ["REVIEW"]
PROBE = REVIEW + "/catalog/attic/extattr_walk"
FIXTURES = REVIEW + "/catalog/fixtures/"


def die(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)


def dir_stats(path):
    t0 = time.perf_counter()
    r = subprocess.run([APFS_UTIL, "-S", path], capture_output=True, text=True)
    ms = (time.perf_counter() - t0) * 1000
    out = {"ms": round(ms, 2), "rc": r.returncode}
    for key, pat in (("descendants", r"descendants:\s*(\d+)"), ("physical_size", r"physical size:\s*(\d+)"), ("gen_count", r"gen-count:\s*(\d+)")):
        m = re.search(pat, r.stdout)
        out[key] = int(m.group(1)) if m else None
    if r.returncode != 0:
        out["stderr"] = r.stderr.strip()
    return out


def gencount(path):
    r = subprocess.run([PROBE, "--probe", path], capture_output=True, text=True)
    m = re.search(r"recursive_gencount=(\d+)", r.stdout)
    return int(m.group(1)) if m else None


def snapshot(root, extra=()):
    snap = {"root": dir_stats(root), "root_gencount": gencount(root)}
    for p in extra:
        snap[os.path.relpath(p, root)] = {"stats": dir_stats(p), "gencount": gencount(p)}
    return snap


def populate(root):
    if not os.path.isdir(root):
        die("root must exist and be enabled")
    for a in range(10):
        for b in range(5):
            d = os.path.join(root, f"d{a:02d}", f"s{b}")
            os.makedirs(d)
            for f in range(20):
                with open(os.path.join(d, f"file-{f:03d}.txt"), "w") as fh:
                    fh.write("payload " * 512)  # 4096 bytes
    deep = os.path.join(root, "deep")
    for i in range(12):
        deep = os.path.join(deep, f"level{i:02d}")
    os.makedirs(deep)
    with open(os.path.join(deep, "leaf.txt"), "w") as fh:
        fh.write("leaf " * 1024)
    os.makedirs(os.path.join(root, "ops"))
    for name in ["append.txt", "truncate.txt", "delete.txt", "rename.txt", "chmod.txt", "xattr.txt", "clone-src.txt",
                 "link-target.txt", "openwriter.txt", "touch.txt", "move-out.txt"]:
        with open(os.path.join(root, "ops", name), "w") as fh:
            fh.write("payload " * 512)
    os.sync()
    print("populated")


def matrix(root, out_path):
    ops_dir = os.path.join(root, "ops")
    deep_leaf = os.path.join(root, *["deep"] + [f"level{i:02d}" for i in range(12)], "leaf.txt")
    deep_dir = os.path.dirname(deep_leaf)
    sub = os.path.join(root, "d03", "s2")
    sibling = os.path.join(os.path.dirname(root), os.path.basename(root) + "-unmaintained")
    os.makedirs(sibling, exist_ok=True)
    extra = [ops_dir, sub, deep_dir, os.path.join(root, "d03")]
    log = []

    def step(name, fn, comment=""):
        before = snapshot(root, extra)
        t0 = time.perf_counter()
        info = fn()
        if not isinstance(info, dict):
            info = {}
        ms = (time.perf_counter() - t0) * 1000
        after = snapshot(root, extra)
        entry = {"step": name, "comment": comment, "op_ms": round(ms, 2), "info": info,
                 "root_gencount_before": before["root_gencount"], "root_gencount_after": after["root_gencount"],
                 "root_before": before["root"], "root_after": after["root"],
                 "sub_gencounts_after": {k: v["gencount"] for k, v in after.items() if k not in ("root", "root_gencount")},
                 "sub_stats_after": {k: v["stats"] for k, v in after.items() if k not in ("root", "root_gencount")}}
        log.append(entry)
        d_gc = after["root_gencount"] - before["root_gencount"]
        d_sz = after["root"]["physical_size"] - before["root"]["physical_size"]
        d_n = after["root"]["descendants"] - before["root"]["descendants"]
        print(f"{name:34s} gencount {before['root_gencount']:>6}->{after['root_gencount']:<6} d={d_gc:<4} size d={d_sz:<8} desc d={d_n:<4} -S {after['root']['ms']:.1f} ms {comment}")

    step("baseline_noop", lambda: None, "no-op: does polling itself change anything?")
    step("sync_only", lambda: os.sync(), "sync")

    def append_close():
        with open(os.path.join(ops_dir, "append.txt"), "a") as fh:
            fh.write("more " * 2048)
    step("append_close", append_close, "append 10 KiB and close")

    def deep_append():
        with open(deep_leaf, "a") as fh:
            fh.write("more " * 2048)
    step("deep_append_close", deep_append, "append at depth 13")

    held = {}
    def open_write_nosync():
        fd = os.open(os.path.join(ops_dir, "openwriter.txt"), os.O_WRONLY | os.O_APPEND)
        os.write(fd, b"open " * 4096)
        held["fd"] = fd
        return {"fstat_size": os.fstat(fd).st_size}
    step("open_writer_write_no_sync", open_write_nosync, "write 20 KiB, fd still open, no fsync")
    step("open_writer_wait_2s", lambda: time.sleep(2.0), "wait 2 s while open")
    step("open_writer_fsync", lambda: os.fsync(held["fd"]), "fsync while open")
    step("open_writer_write_again", lambda: os.write(held["fd"], b"again " * 4096), "second write while open")
    step("open_writer_close", lambda: os.close(held["fd"]), "close")

    step("truncate", lambda: os.truncate(os.path.join(ops_dir, "truncate.txt"), 0), "truncate to 0")
    step("delete", lambda: os.unlink(os.path.join(ops_dir, "delete.txt")), "unlink")
    step("rename_same_dir", lambda: os.rename(os.path.join(ops_dir, "rename.txt"), os.path.join(ops_dir, "renamed.txt")), "rename in place")
    step("rename_across_subdirs", lambda: os.rename(os.path.join(ops_dir, "renamed.txt"), os.path.join(sub, "renamed.txt")), "ops -> d03/s2")
    step("rename_dir_within", lambda: os.rename(os.path.join(root, "d09"), os.path.join(root, "d09-renamed")), "rename a populated dir")
    step("move_out_of_tree", lambda: os.rename(os.path.join(ops_dir, "move-out.txt"), os.path.join(sibling, "move-out.txt")), "to unmaintained sibling")
    step("move_into_tree", lambda: os.rename(os.path.join(sibling, "move-out.txt"), os.path.join(ops_dir, "move-in.txt")), "back from unmaintained sibling")
    step("chmod", lambda: os.chmod(os.path.join(ops_dir, "chmod.txt"), 0o600), "chmod")
    step("touch_mtime_only", lambda: os.utime(os.path.join(ops_dir, "touch.txt"), (1577836800, 1577836800)), "utime")
    step("xattr_set", lambda: subprocess.run(["xattr", "-w", "com.example.k", "v", os.path.join(ops_dir, "xattr.txt")], check=True), "xattr")
    step("hardlink_add", lambda: os.link(os.path.join(ops_dir, "link-target.txt"), os.path.join(ops_dir, "link-alias.txt")), "hard link in same dir")
    step("hardlink_add_other_dir", lambda: os.link(os.path.join(ops_dir, "link-target.txt"), os.path.join(sub, "link-alias2.txt")), "hard link into d03/s2")
    step("clone_cp_c", lambda: subprocess.run(["cp", "-c", os.path.join(ops_dir, "clone-src.txt"), os.path.join(ops_dir, "clone-dst.txt")], check=True), "cp -c clone")
    step("symlink", lambda: os.symlink("append.txt", os.path.join(ops_dir, "sym")), "symlink")
    step("mkdir_new", lambda: os.makedirs(os.path.join(ops_dir, "newdir", "inner")), "mkdir -p")
    def create_in_new():
        with open(os.path.join(ops_dir, "newdir", "inner", "f.txt"), "w") as fh:
            fh.write("x" * 100000)
    step("create_in_new_dir", create_in_new, "100000-byte file in new subdir")
    step("rmtree_subdir", lambda: subprocess.run(["rm", "-rf", os.path.join(root, "d08")], check=True), "rm -rf d08 (100 files)")
    def big_sparse():
        p = os.path.join(ops_dir, "sparse.bin")
        with open(p, "wb") as fh:
            fh.seek(50 * 1024 * 1024)
            fh.write(b"end")
    step("sparse_file", big_sparse, "50 MiB sparse file: physical vs logical")
    step("read_only_stat", lambda: os.lstat(deep_leaf), "lstat only")
    # Nested origin: enable on a subdirectory that is already inside the maintained tree.
    def nested_origin():
        r = subprocess.run([APFS_UTIL, "-M", os.path.join(root, "d03")], capture_output=True, text=True)
        return {"rc": r.returncode, "out": r.stdout.strip(), "err": r.stderr.strip()}
    step("nested_origin_enable_d03", nested_origin, "apfs.util -M on non-empty d03 inside tree")
    def append_under_nested():
        with open(os.path.join(sub, "file-000.txt"), "a") as fh:
            fh.write("nested " * 1024)
    step("append_under_nested_origin", append_under_nested, "append in d03/s2: does d03's gencount move? root's?")
    def move_populated_in():
        src = os.path.join(sibling, "populated")
        for i in range(3):
            os.makedirs(os.path.join(src, f"p{i}"))
            for f in range(30):
                with open(os.path.join(src, f"p{i}", f"f{f}.txt"), "w") as fh:
                    fh.write("z" * 8192)
        os.rename(src, os.path.join(root, "moved-in-populated"))
    step("move_populated_unmaintained_dir_in", move_populated_in, "90 x 8 KiB files (737280 B) moved in from an unmaintained dir: immediate or 'calculating'?")
    step("recheck_after_1s", lambda: time.sleep(1.0), "re-read totals after 1 s")
    def deep_chain():
        d = os.path.join(root, "chain")
        for i in range(120):
            d = os.path.join(d, f"c{i:03d}")
        os.makedirs(d)
        with open(os.path.join(d, "leaf.txt"), "w") as fh:
            fh.write("q" * 12288)
        return {"depth": 121}
    step("deep_chain_121_create", deep_chain, "121-level chain + 12 KiB leaf: chain-too-long limit?")
    def deep_chain_append():
        d = os.path.join(root, "chain")
        for i in range(120):
            d = os.path.join(d, f"c{i:03d}")
        with open(os.path.join(d, "leaf.txt"), "a") as fh:
            fh.write("q" * 12288)
    step("deep_chain_121_append", deep_chain_append, "append 12 KiB at depth 121")
    with open(out_path, "w") as fh:
        json.dump(log, fh, indent=1)
    print(f"wrote {out_path}")


def main():
    if len(sys.argv) < 3:
        die(__doc__)
    cmd, root = sys.argv[1], os.path.abspath(sys.argv[2])
    if not root.startswith(FIXTURES):
        die("root must be under the catalog fixtures directory")
    if cmd == "populate":
        populate(root)
    elif cmd == "matrix":
        matrix(root, sys.argv[3])
    elif cmd == "stats":
        print(json.dumps({"stats": dir_stats(root), "gencount": gencount(root)}))
    else:
        die(__doc__)


if __name__ == "__main__":
    main()
