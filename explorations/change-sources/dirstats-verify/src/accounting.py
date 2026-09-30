#!/usr/bin/env python3
"""What the dir-stats "physical size" counts, versus st_blocks*512, ATTR_FILE_ALLOCSIZE,
ATTR_FILE_DATAALLOCSIZE and ATTR_CMNEXT_PRIVATESIZE, for clones, hard links, sparse files,
delayed allocation (open writer before fsync), resource forks and large xattrs.

  accounting.py PARENT OUT.json

Builds PARENT/acct/root (marked) and PARENT/acct/outside. Each case records the root
fsctl totals delta and the per-file numbers from lstat and getattrlist.
"""
import json
import os
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dsapi  # noqa: E402
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402


def fileinfo(path):
    st = os.lstat(path)
    g = dsapi.gen(path)
    return {"st_size": st.st_size, "st_blocks512": st.st_blocks * 512, "nlink": st.st_nlink, "ino": st.st_ino,
            "alloc": g["alloc"], "dataalloc": g["dataalloc"], "rsrcalloc": g["rsrcalloc"], "datalen": g["datalen"],
            "privatesize": g["privatesize"], "cloneid": g["cloneid"], "refcnt": g["refcnt"], "extflags": hex(g["extflags"] or 0)}


def main():
    parent, out = os.path.abspath(sys.argv[1]), sys.argv[2]
    base = os.path.join(parent, "acct")
    if os.path.exists(base):
        shutil.rmtree(base)
    root = os.path.join(base, "root"); outside = os.path.join(base, "outside")
    os.makedirs(root); os.makedirs(outside)
    assert dsapi.apfs_util_M(root)["rc"] == 0
    log = []

    def case(name, fn, files):
        b = dsapi.get(root)
        info = fn() or {}
        a = dsapi.get(root)
        rec = {"case": name, "delta": (a["gen"] - b["gen"], a["desc"] - b["desc"], a["phys"] - b["phys"]), "root_phys": a["phys"], "root_desc": a["desc"],
               "files": {os.path.relpath(f, base): fileinfo(f) for f in files if os.path.lexists(f)}, "info": info}
        log.append(rec)
        print(name, rec["delta"], json.dumps(rec["files"]), info)

    def mk(path, size):
        with open(path, "wb") as fh:
            fh.write(b"q" * size)
        return path

    # 1. plain file 100 KiB
    p1 = os.path.join(root, "plain.bin")
    case("plain_100k", lambda: mk(p1, 102400), [p1])
    # 2. clone inside tree
    p2 = os.path.join(root, "clone.bin")
    case("clone_inside", lambda: subprocess.run(["cp", "-c", p1, p2], check=True), [p1, p2])
    # 3. clone of an inside file to outside
    p3 = os.path.join(outside, "clone-out.bin")
    case("clone_to_outside", lambda: subprocess.run(["cp", "-c", p1, p3], check=True), [p1, p3])
    # 4. write 1 byte into the clone (breaks sharing for one block)
    def poke():
        fd = os.open(p2, os.O_WRONLY); os.pwrite(fd, b"Z", 0); os.close(fd)
    case("clone_write_1B", poke, [p1, p2])
    # 5. hard link inside (second name)
    p5 = os.path.join(root, "sub"); os.makedirs(p5)
    l5 = os.path.join(p5, "plain-link.bin")
    case("hardlink_inside", lambda: os.link(p1, l5), [p1, l5])
    # 6. hard link to outside
    l6 = os.path.join(outside, "plain-outlink.bin")
    case("hardlink_to_outside", lambda: os.link(p1, l6), [p1, l6])
    # 7. remove inside link
    case("hardlink_inside_rm", lambda: os.unlink(l5), [p1])
    # 8. sparse 50 MiB
    p8 = os.path.join(root, "sparse.bin")
    def sparse():
        with open(p8, "wb") as fh:
            fh.seek(50 * 1024 * 1024); fh.write(b"e")
    case("sparse_50M", sparse, [p8])
    # 9. sparse small (1 MiB)
    p9 = os.path.join(root, "sparse-small.bin")
    def sparse_small():
        with open(p9, "wb") as fh:
            fh.seek(1024 * 1024); fh.write(b"e")
    case("sparse_1M", sparse_small, [p9])
    # 10. open writer 256 KiB, before fsync; then after fsync; then after close
    p10 = os.path.join(root, "openwriter.log")
    held = {}
    def ow():
        fd = os.open(p10, os.O_WRONLY | os.O_CREAT, 0o644); held["fd"] = fd
        os.write(fd, b"L" * 262144)
        return {"fstat_blocks512": os.fstat(fd).st_blocks * 512, "fstat_size": os.fstat(fd).st_size}
    case("openwriter_256k_nosync", ow, [p10])
    case("openwriter_fsync", lambda: os.fsync(held["fd"]), [p10])
    case("openwriter_close", lambda: os.close(held["fd"]), [p10])
    # 11. resource fork 64 KiB and a 64 KiB xattr
    p11 = mk(os.path.join(root, "rsrc.bin"), 4096)
    case("rsrc_fork_64k", lambda: subprocess.run(["xattr", "-w", "com.apple.ResourceFork", "R" * 65536, p11], check=True), [p11])
    p12 = mk(os.path.join(root, "xattr.bin"), 4096)
    case("xattr_64k", lambda: subprocess.run(["xattr", "-w", "com.example.big", "X" * 65536, p12], check=True), [p12])
    p12b = mk(os.path.join(root, "xattr-small.bin"), 4096)
    case("xattr_2k", lambda: subprocess.run(["xattr", "-w", "com.example.small", "X" * 2048, p12b], check=True), [p12b])
    # 12. compressed file (ditto --hfsCompression), if available
    p13 = os.path.join(root, "compressed.txt")
    def compress():
        src = mk(os.path.join(outside, "text.txt"), 0)
        with open(src, "w") as fh:
            fh.write("compressible text " * 20000)
        r = subprocess.run(["ditto", "--hfsCompression", src, p13], capture_output=True, text=True)
        return {"rc": r.returncode, "err": r.stderr.strip()[:200]}
    case("hfs_compressed_copy", compress, [p13])
    p13b = os.path.join(root, "compressed-2m.txt")
    def compress_big():
        src = os.path.join(outside, "text2m.txt")
        with open(src, "w") as fh:
            fh.write("compressible text " * 120000)
        r = subprocess.run(["ditto", "--hfsCompression", src, p13b], capture_output=True, text=True)
        return {"rc": r.returncode, "err": r.stderr.strip()[:200], "xattrs": subprocess.run(["xattr", "-l", p13b], capture_output=True, text=True).stdout[:200]}
    case("hfs_compressed_copy_2m", compress_big, [p13b])
    # 13. symlink and empty dir
    p14 = os.path.join(root, "sym"); p15 = os.path.join(root, "emptydir")
    case("symlink", lambda: os.symlink("plain.bin", p14), [p14])
    case("empty_dir", lambda: os.makedirs(p15), [p15])
    # 14. truncate the clone partner p1 to 0: does phys of the still-shared p2 change?
    case("truncate_clone_src", lambda: os.truncate(p1, 0), [p1, p2])
    # final: walker totals vs fsctl
    ds = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bin", "ds")
    r = subprocess.run([ds, "walk", root, "-o", "/dev/null"], capture_output=True, text=True)
    t = dsapi.get(root)
    log.append({"case": "final_totals", "fsctl": (t["gen"], t["desc"], t["phys"]), "walk_summary": r.stdout.strip()})
    print(log[-1])
    write_text_atomic(out, json.dumps(log, indent=1, default=str))


if __name__ == "__main__":
    main()
