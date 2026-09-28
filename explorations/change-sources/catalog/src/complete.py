#!/usr/bin/env python3
# Ported: the fixture guard uses REVIEW from the environment instead of an absolute scratch path.
"""Completeness harness for searchfs(2) change-time queries against an lstat oracle.

Subcommands (stdlib only; run with `uv run --no-project python complete.py ...`):
  prepare FIXTURE            create the baseline tree, sleep past the second boundary,
                             record t0 (epoch seconds) in FIXTURE/../<name>.t0
  mutate FIXTURE             apply the operation matrix; write expected effects JSON
  openwriter FIXTURE SEARCHFS VOL OUT   hold an fd open across an append+fsync and run
                             searchfs (ctime and mtime windows) while open, then after close
  compare FIXTURE T0 DUMP_CTIME DUMP_MTIME OUT_JSON
                             lstat oracle vs searchfs dumps; per-op verdicts

Never point this at a real tree: it mutates FIXTURE.
"""
import json
import os
import shutil
import stat
import subprocess
import sys
import time

REVIEW = os.environ["REVIEW"]


def die(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)


def t0_path(fixture):
    return os.path.join(os.path.dirname(fixture.rstrip("/")), os.path.basename(fixture.rstrip("/")) + ".t0")


def prepare(fixture):
    if os.path.exists(fixture):
        die(f"refusing existing fixture {fixture}")
    os.makedirs(fixture)
    # Baseline: 20 dirs x 25 files, plus dedicated targets for each operation.
    for d in range(20):
        dd = os.path.join(fixture, f"base-{d:02d}")
        os.makedirs(dd)
        for f in range(25):
            with open(os.path.join(dd, f"file-{f:03d}.txt"), "w") as fh:
                fh.write("baseline " * 64)
    ops = os.path.join(fixture, "ops")
    os.makedirs(os.path.join(ops, "srcdir"))
    os.makedirs(os.path.join(ops, "dstdir"))
    os.makedirs(os.path.join(ops, "rename-me-dir", "inner"))
    with open(os.path.join(ops, "rename-me-dir", "inner", "child.txt"), "w") as fh:
        fh.write("child " * 100)
    with open(os.path.join(ops, "rename-me-dir", "top.txt"), "w") as fh:
        fh.write("top " * 100)
    for name in [
        "overwrite.txt", "append.txt", "truncate.txt", "rename-same.txt", "chmod.txt", "xattr.txt",
        "clone-src.txt", "touch-old.txt", "utimes-old.txt", "link-target.txt", "unlink-one.txt",
        "delete-me.txt", "untouched.txt", "openwriter.txt", "write-via-link.txt", "cp-p-src.txt",
    ]:
        with open(os.path.join(ops, name), "w") as fh:
            fh.write(name + " " + "payload " * 200)
    with open(os.path.join(ops, "srcdir", "move-across.txt"), "w") as fh:
        fh.write("move " * 100)
    os.link(os.path.join(ops, "unlink-one.txt"), os.path.join(ops, "unlink-one-alias.txt"))
    os.link(os.path.join(ops, "write-via-link.txt"), os.path.join(ops, "write-via-link-alias.txt"))
    os.sync()
    # Make sure every baseline ctime is strictly below t0 at one-second granularity.
    time.sleep(2.2)
    t0 = int(time.time())
    time.sleep(1.1)
    with open(t0_path(fixture), "w") as fh:
        fh.write(str(t0))
    print(f"prepared {fixture} t0={t0}")


def mutate(fixture):
    ops = os.path.join(fixture, "ops")
    expected = {}

    def note(op, path, changed_bytes, expect_ctime, expect_mtime, comment=""):
        expected[op] = {"path": os.path.relpath(path, fixture), "bytes_changed": changed_bytes,
                        "expect_ctime_new": expect_ctime, "expect_mtime_new": expect_mtime, "comment": comment}

    p = os.path.join(ops, "new-file.txt")
    with open(p, "w") as fh:
        fh.write("new " * 500)
    note("create", p, True, True, True)
    p = os.path.join(ops, "new-dir")
    os.makedirs(p)
    with open(os.path.join(p, "inside.txt"), "w") as fh:
        fh.write("inside " * 500)
    note("mkdir", p, True, True, True)
    note("mkdir_child", os.path.join(p, "inside.txt"), True, True, True)
    p = os.path.join(ops, "overwrite.txt")
    with open(p, "r+b") as fh:
        fh.seek(0)
        fh.write(b"OVERWRITE")
    note("overwrite_inplace", p, True, True, True, "same length, bytes changed")
    p = os.path.join(ops, "append.txt")
    with open(p, "a") as fh:
        fh.write("appended " * 1000)
    note("append_close", p, True, True, True)
    p = os.path.join(ops, "truncate.txt")
    os.truncate(p, 0)
    note("truncate", p, True, True, True)
    src = os.path.join(ops, "rename-same.txt")
    dst = os.path.join(ops, "rename-same-renamed.txt")
    os.rename(src, dst)
    note("rename_same_dir", dst, False, None, False, "does the moved object's ctime move? recorded, not asserted")
    src = os.path.join(ops, "srcdir", "move-across.txt")
    dst = os.path.join(ops, "dstdir", "move-across.txt")
    os.rename(src, dst)
    note("rename_across_dirs", dst, False, None, False)
    note("rename_across_src_parent", os.path.join(ops, "srcdir"), False, True, True)
    note("rename_across_dst_parent", os.path.join(ops, "dstdir"), False, True, True)
    src = os.path.join(ops, "rename-me-dir")
    dst = os.path.join(ops, "renamed-dir")
    os.rename(src, dst)
    note("rename_dir", dst, False, None, False, "renamed directory itself")
    note("rename_dir_child", os.path.join(dst, "inner", "child.txt"), False, False, False, "descendant of renamed dir: expect unchanged")
    note("rename_dir_inner", os.path.join(dst, "inner"), False, False, False)
    p = os.path.join(ops, "delete-me.txt")
    os.unlink(p)
    note("delete", p, True, None, None, "object gone; only the parent can show it")
    note("delete_parent", ops, False, True, True)
    p = os.path.join(ops, "link-target.txt")
    os.link(p, os.path.join(ops, "link-target-alias.txt"))
    note("hardlink_add_target", p, False, None, False, "link count changed on target")
    p = os.path.join(ops, "unlink-one.txt")
    os.unlink(os.path.join(ops, "unlink-one-alias.txt"))
    note("hardlink_remove_survivor", p, False, None, False)
    p = os.path.join(ops, "write-via-link-alias.txt")
    with open(p, "a") as fh:
        fh.write("via alias " * 300)
    note("write_via_hardlink_alias", os.path.join(ops, "write-via-link.txt"), True, True, True, "same inode as alias")
    p = os.path.join(ops, "chmod.txt")
    os.chmod(p, 0o600)
    note("chmod", p, False, True, False)
    p = os.path.join(ops, "xattr.txt")
    subprocess.run(["xattr", "-w", "com.example.probe", "value", p], check=True)
    note("xattr_set", p, False, None, False, "xattr set: does ctime move?")
    src = os.path.join(ops, "clone-src.txt")
    dst = os.path.join(ops, "clone-dst.txt")
    subprocess.run(["cp", "-c", src, dst], check=True)
    note("clone_cp_c", dst, True, None, None, "new clone object: cp -c may preserve nothing or mtime")
    note("clone_src_after", src, False, False, False)
    p = os.path.join(ops, "touch-old.txt")
    subprocess.run(["touch", "-t", "202001010000", p], check=True)
    note("touch_old_mtime", p, False, True, False, "mtime set to 2020; ctime should be now")
    p = os.path.join(ops, "utimes-old.txt")
    os.utime(p, (1577836800, 1577836800))
    note("utime_old", p, False, True, False)
    src = os.path.join(ops, "cp-p-src.txt")
    dst = os.path.join(ops, "cp-p-dst.txt")
    subprocess.run(["cp", "-p", src, dst], check=True)
    note("cp_p_preserved_mtime", dst, True, True, False, "new object with old mtime; ctime should be now")
    # tar -p extraction with old mtimes
    tarball = os.path.join(fixture, "..", os.path.basename(fixture) + "-old.tar")
    old_src = os.path.join(ops, "tar-src")
    os.makedirs(old_src)
    with open(os.path.join(old_src, "old-file.txt"), "w") as fh:
        fh.write("tarred " * 300)
    os.utime(os.path.join(old_src, "old-file.txt"), (1577836800, 1577836800))
    os.utime(old_src, (1577836800, 1577836800))
    subprocess.run(["tar", "-cf", tarball, "-C", ops, "tar-src"], check=True)
    extract = os.path.join(ops, "tar-extract")
    os.makedirs(extract)
    subprocess.run(["tar", "-xpf", tarball, "-C", extract], check=True)
    note("tar_extract_old_mtime", os.path.join(extract, "tar-src", "old-file.txt"), True, True, False)
    note("tar_extract_old_dir", os.path.join(extract, "tar-src"), False, True, False)
    p = os.path.join(ops, "new-symlink")
    os.symlink("untouched.txt", p)
    note("symlink_create", p, False, True, True)
    note("untouched", os.path.join(ops, "untouched.txt"), False, False, False)
    note("untouched_base_dir", os.path.join(fixture, "base-05"), False, False, False)
    note("untouched_base_file", os.path.join(fixture, "base-05", "file-005.txt"), False, False, False)
    os.sync()
    with open(os.path.join(os.path.dirname(fixture.rstrip("/")), os.path.basename(fixture.rstrip("/")) + ".expected.json"), "w") as fh:
        json.dump(expected, fh, indent=1)
    print(f"mutated {fixture}: {len(expected)} expectations")


def oracle(fixture):
    """lstat walk: relpath -> (ino, ctime, mtime, size, blocks, kind, nlink)."""
    out = {}
    for root, dirs, files in os.walk(fixture, followlinks=False):
        for name in dirs + files:
            p = os.path.join(root, name)
            try:
                s = os.lstat(p)
            except FileNotFoundError:
                continue
            out[os.path.relpath(p, fixture)] = {
                "ino": s.st_ino, "ctime": s.st_ctime, "mtime": s.st_mtime, "size": s.st_size,
                "blocks": s.st_blocks, "nlink": s.st_nlink,
                "kind": "dir" if stat.S_ISDIR(s.st_mode) else "link" if stat.S_ISLNK(s.st_mode) else "file",
            }
    s = os.lstat(fixture)
    out["."] = {"ino": s.st_ino, "ctime": s.st_ctime, "mtime": s.st_mtime, "size": s.st_size,
                "blocks": s.st_blocks, "nlink": s.st_nlink, "kind": "dir"}
    return out


def load_dump(path):
    rows = {}
    with open(path) as fh:
        header = fh.readline().rstrip("\n").split("\t")
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < len(header):
                continue
            rec = dict(zip(header, parts))
            rows.setdefault(int(rec["fileid"]), []).append(rec)
    return rows


def compare(fixture, t0, dump_ctime, dump_mtime, out_json):
    t0 = int(t0)
    orc = oracle(fixture)
    exp_path = os.path.join(os.path.dirname(fixture.rstrip("/")), os.path.basename(fixture.rstrip("/")) + ".expected.json")
    expected = json.load(open(exp_path)) if os.path.exists(exp_path) else {}
    ids = {v["ino"] for v in orc.values()}
    dc = load_dump(dump_ctime)
    dm = load_dump(dump_mtime)
    found_c = {i for i in dc if i in ids}
    found_m = {i for i in dm if i in ids}
    oracle_c = {v["ino"] for v in orc.values() if v["ctime"] >= t0}
    oracle_m = {v["ino"] for v in orc.values() if v["mtime"] >= t0}
    by_ino = {}
    for rel, v in orc.items():
        by_ino.setdefault(v["ino"], []).append(rel)
    result = {
        "t0": t0,
        "oracle_entries": len(orc),
        "oracle_ctime_ge_t0": len(oracle_c),
        "oracle_mtime_ge_t0": len(oracle_m),
        "searchfs_ctime_hits_in_fixture": len(found_c),
        "searchfs_mtime_hits_in_fixture": len(found_m),
        "ctime_missing_from_searchfs": sorted(by_ino[i][0] for i in oracle_c - found_c),
        "ctime_extra_in_searchfs": sorted(by_ino[i][0] for i in found_c - oracle_c),
        "mtime_missing_from_searchfs": sorted(by_ino[i][0] for i in oracle_m - found_m),
        "mtime_extra_in_searchfs": sorted(by_ino[i][0] for i in found_m - oracle_m),
        "hardlink_rows_per_ino": {str(i): len(dc[i]) for i in found_c if len(dc[i]) > 1},
        "ops": {},
    }
    for op, e in expected.items():
        rel = e["path"]
        v = orc.get(rel)
        row = {"path": rel, "bytes_changed": e["bytes_changed"], "comment": e["comment"]}
        if v is None:
            row["oracle"] = "absent"
        else:
            row["ctime_new"] = v["ctime"] >= t0
            row["mtime_new"] = v["mtime"] >= t0
            row["searchfs_ctime_found"] = v["ino"] in found_c
            row["searchfs_mtime_found"] = v["ino"] in found_m
            row["expect_ctime_new"] = e["expect_ctime_new"]
            row["expect_mtime_new"] = e["expect_mtime_new"]
            row["ctime_matches_expectation"] = (e["expect_ctime_new"] is None) or (row["ctime_new"] == e["expect_ctime_new"])
            row["searchfs_ctime_consistent_with_stat"] = row["searchfs_ctime_found"] == row["ctime_new"]
            row["searchfs_mtime_consistent_with_stat"] = row["searchfs_mtime_found"] == row["mtime_new"]
        result["ops"][op] = row
    with open(out_json, "w") as fh:
        json.dump(result, fh, indent=1)
    print(json.dumps({k: v for k, v in result.items() if k != "ops"}, indent=1))
    for op, row in result["ops"].items():
        print(f"{op:32s} ctime_new={row.get('ctime_new')!s:5} mtime_new={row.get('mtime_new')!s:5} "
              f"sfs_ctime={row.get('searchfs_ctime_found')!s:5} sfs_mtime={row.get('searchfs_mtime_found')!s:5} "
              f"consistent={row.get('searchfs_ctime_consistent_with_stat')!s:5} {row.get('oracle', '')} {row['comment']}")


def openwriter(fixture, searchfs_bin, vol, outdir):
    """Hold a descriptor open across an append + fsync and query searchfs while open."""
    os.makedirs(outdir, exist_ok=True)
    p = os.path.join(fixture, "ops", "openwriter.txt")
    before = os.lstat(p)
    time.sleep(2.2)
    t_before = int(time.time())
    time.sleep(1.1)
    fd = os.open(p, os.O_WRONLY | os.O_APPEND)
    os.write(fd, b"open-writer append " * 2000)
    os.fsync(fd)
    st_open = os.fstat(fd)
    st_open_path = os.lstat(p)
    summary = {
        "t_before": t_before,
        "ctime_before": before.st_ctime, "mtime_before": before.st_mtime, "size_before": before.st_size,
        "fstat_while_open": {"ctime": st_open.st_ctime, "mtime": st_open.st_mtime, "size": st_open.st_size},
        "lstat_while_open": {"ctime": st_open_path.st_ctime, "mtime": st_open_path.st_mtime, "size": st_open_path.st_size},
        "ino": st_open.st_ino,
    }
    runs = {}
    attrs = ("ctime",) if os.environ.get("OPENWRITER_CTIME_ONLY") else ("ctime", "mtime")
    for attr in attrs:
        dump = os.path.join(outdir, f"open-{attr}.tsv")
        r = subprocess.run([searchfs_bin, vol, attr, str(t_before), str(int(time.time()) + 5), "--dump", dump, "--label", f"openwriter-open-{attr}"],
                           capture_output=True, text=True)
        rows = load_dump(dump) if os.path.exists(dump) else {}
        runs[f"open_{attr}"] = {"summary": r.stdout.strip(), "stderr": r.stderr.strip(), "found": st_open.st_ino in rows,
                                "row": rows.get(st_open.st_ino)}
    os.close(fd)
    st_closed = os.lstat(p)
    summary["lstat_after_close"] = {"ctime": st_closed.st_ctime, "mtime": st_closed.st_mtime, "size": st_closed.st_size}
    for attr in attrs:
        dump = os.path.join(outdir, f"closed-{attr}.tsv")
        r = subprocess.run([searchfs_bin, vol, attr, str(t_before), str(int(time.time()) + 5), "--dump", dump, "--label", f"openwriter-closed-{attr}"],
                           capture_output=True, text=True)
        rows = load_dump(dump) if os.path.exists(dump) else {}
        runs[f"closed_{attr}"] = {"summary": r.stdout.strip(), "stderr": r.stderr.strip(), "found": st_open.st_ino in rows,
                                  "row": rows.get(st_open.st_ino)}
    summary["runs"] = runs
    with open(os.path.join(outdir, "openwriter.json"), "w") as fh:
        json.dump(summary, fh, indent=1)
    print(json.dumps(summary, indent=1))


def main():
    if len(sys.argv) < 3:
        die(__doc__)
    cmd, fixture = sys.argv[1], os.path.abspath(sys.argv[2])
    if not fixture.startswith(REVIEW + "/catalog/fixtures/"):
        die("fixture must be under the review catalog fixtures directory")
    if cmd == "prepare":
        prepare(fixture)
    elif cmd == "mutate":
        mutate(fixture)
    elif cmd == "openwriter":
        openwriter(fixture, sys.argv[3], sys.argv[4], sys.argv[5])
    elif cmd == "compare":
        compare(fixture, sys.argv[3], sys.argv[4], sys.argv[5], sys.argv[6])
    else:
        die(__doc__)


if __name__ == "__main__":
    main()
