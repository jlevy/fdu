#!/usr/bin/env python3
"""Controlled open-writer experiments against a live FSEvents stream plus short-cursor replay.

Each trial: create a fresh trial directory and target file, let it settle, start `livewatch`
(sinceNow, FileEvents, NoDefer, latency 0) on that directory, perform an operation while the
descriptor stays open for HOLD seconds (flushing the stream and sampling stat along the way),
run a historical replay (`probe replay`) from the pre-operation device fence while still open,
close, flush again, replay again, and record everything as one JSON object per line.

Only synthetic fixtures under the caller-supplied --root are touched.
"""

from __future__ import annotations

import argparse
import ctypes
import ctypes.util
import fcntl
import json
import os
import sqlite3
import struct
import subprocess
import sys
import threading
import time
from pathlib import Path
from typing import Any

libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)
libc.mmap.restype = ctypes.c_void_p
libc.mmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_longlong]
libc.munmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
libc.msync.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
libc.clonefile.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint32]
PROT_READ, PROT_WRITE, MAP_SHARED, MS_SYNC = 1, 2, 1, 0x10
F_PREALLOCATE, F_ALLOCATECONTIG, F_ALLOCATEALL, F_PEOFPOSMODE = 42, 2, 4, 3
F_FULLFSYNC = 51

FLAG_NAMES = {
    0x1: "MustScanSubDirs", 0x2: "UserDropped", 0x4: "KernelDropped", 0x8: "EventIdsWrapped",
    0x10: "HistoryDone", 0x20: "RootChanged", 0x40: "Mount", 0x80: "Unmount",
    0x100: "ItemCreated", 0x200: "ItemRemoved", 0x400: "ItemInodeMetaMod", 0x800: "ItemRenamed",
    0x1000: "ItemModified", 0x2000: "ItemFinderInfoMod", 0x4000: "ItemChangeOwner",
    0x8000: "ItemXattrMod", 0x10000: "ItemIsFile", 0x20000: "ItemIsDir", 0x40000: "ItemIsSymlink",
    0x80000: "OwnEvent", 0x100000: "ItemIsHardlink", 0x200000: "ItemIsLastHardlink",
    0x400000: "ItemCloned",
}


def flag_names(flags: int) -> str:
    names = [name for bit, name in FLAG_NAMES.items() if flags & bit]
    rest = flags & ~sum(FLAG_NAMES)
    if rest:
        names.append(hex(rest))
    return "|".join(names) or "0"


def stat_snapshot(path: str | None, fd: int | None) -> dict[str, Any]:
    out: dict[str, Any] = {}
    try:
        st = os.fstat(fd) if fd is not None else os.stat(path)  # type: ignore[arg-type]
        out = {
            "size": st.st_size, "blocks": st.st_blocks, "nlink": st.st_nlink, "ino": st.st_ino,
            "mtime_ns": st.st_mtime_ns, "ctime_ns": st.st_ctime_ns,
        }
        if path is not None and fd is not None:
            try:
                ps = os.stat(path)
                out["path_stat_matches_fstat"] = (ps.st_size, ps.st_blocks, ps.st_mtime_ns, ps.st_ctime_ns) == (
                    st.st_size, st.st_blocks, st.st_mtime_ns, st.st_ctime_ns)
                out["path_ino"] = ps.st_ino
            except FileNotFoundError:
                out["path_stat_matches_fstat"] = None
    except OSError as error:
        out = {"error": error.errno}
    return out


def statvfs_avail(path: str) -> int:
    vfs = os.statvfs(path)
    return vfs.f_bavail * vfs.f_frsize


class Watcher:
    def __init__(self, binary: Path, root: Path) -> None:
        self.proc = subprocess.Popen(
            [str(binary), str(root)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, bufsize=1,
        )
        self.records: list[dict[str, Any]] = []
        self.lock = threading.Lock()
        self.cv = threading.Condition(self.lock)
        self.thread = threading.Thread(target=self._reader, daemon=True)
        self.thread.start()
        self.ready = self._wait_for(lambda r: r.get("type") == "ready", 10.0)
        if self.ready is None:
            raise RuntimeError("watcher never became ready: " + (self.proc.stderr.read() if self.proc.stderr else ""))

    def _reader(self) -> None:
        assert self.proc.stdout is not None
        for line in self.proc.stdout:
            try:
                record = json.loads(line)
            except ValueError:
                record = {"type": "malformed", "line": line}
            with self.cv:
                self.records.append(record)
                self.cv.notify_all()

    def _wait_for(self, predicate, timeout: float) -> dict[str, Any] | None:
        deadline = time.monotonic() + timeout
        with self.cv:
            while True:
                for record in reversed(self.records):
                    if predicate(record):
                        return record
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return None
                self.cv.wait(remaining)

    def command(self, text: str, expect: str, key: str | None = None, value: str | None = None) -> dict[str, Any]:
        assert self.proc.stdin is not None
        before = len(self.records)
        self.proc.stdin.write(text + "\n")
        self.proc.stdin.flush()
        record = self._wait_for(
            lambda r: r.get("type") == expect and (key is None or r.get(key) == value)
            and self.records.index(r) >= before, 30.0)
        if record is None:
            raise RuntimeError(f"watcher did not answer {text!r}")
        return record

    def mark(self, label: str) -> dict[str, Any]:
        return self.command(f"mark {label}", "mark", "label", label)

    def flush(self) -> dict[str, Any]:
        return self.command("flush", "flushed")

    def close(self) -> dict[str, Any]:
        assert self.proc.stdin is not None
        self.proc.stdin.write("quit\n")
        self.proc.stdin.flush()
        self.proc.wait(timeout=30)
        self.thread.join(timeout=5)
        summaries = [r for r in self.records if r.get("type") == "summary"]
        return summaries[-1] if summaries else {}


def replay(probe: Path, root: Path, cursor: int, seconds: int = 10) -> dict[str, Any]:
    started = time.monotonic()
    try:
        result = subprocess.run(
            [str(probe), "replay", str(root), str(cursor), "16", str(seconds), "wait"],
            capture_output=True, text=True, timeout=seconds + 10, check=False)
        stdout, returncode, killed = result.stdout, result.returncode, False
    except subprocess.TimeoutExpired as error:
        stdout = (error.stdout or b"").decode(errors="replace") if isinstance(error.stdout, bytes) else (error.stdout or "")
        returncode, killed = None, True
    records = []
    for line in stdout.splitlines():
        try:
            records.append(json.loads(line))
        except ValueError:
            records.append({"type": "malformed", "line": line})
    events = []
    for record in records:
        if record.get("type") == "event" and not record["flags"] & 0x10:
            rel = bytes.fromhex(record.get("path_hex", "")).decode(errors="replace")
            events.append({"id": record["id"], "flags": record["flags"], "names": flag_names(record["flags"]),
                           "rel": rel, "inside": record.get("inside")})
    summaries = [r for r in records if r.get("type") == "summary"]
    return {
        "cursor": cursor, "events": events, "summary": summaries[-1] if summaries else None,
        "returncode": returncode, "killed": killed, "wall_seconds": round(time.monotonic() - started, 4),
    }


class Trial:
    def __init__(self, args: argparse.Namespace, op: str, rep: int) -> None:
        self.args = args
        self.op = op
        self.rep = rep
        self.dir = Path(args.root) / f"{op}-r{rep}"
        self.dir.mkdir(parents=True)
        self.target = self.dir / "target"
        self.log: list[dict[str, Any]] = []
        self.watcher: Watcher | None = None
        self.stats: list[dict[str, Any]] = []
        self.t0 = time.monotonic()

    def note(self, label: str, **fields: Any) -> None:
        self.log.append({"t": round(time.monotonic() - self.t0, 4), "label": label, **fields})

    def sample(self, label: str, fd: int | None = None, path: str | None = None) -> None:
        self.stats.append({"label": label, "t": round(time.monotonic() - self.t0, 4),
                           "stat": stat_snapshot(path if path is not None else str(self.target), fd)})

    def mark(self, label: str) -> None:
        assert self.watcher is not None
        self.watcher.mark(label)
        self.note("mark", mark=label)

    def flush(self, label: str) -> None:
        assert self.watcher is not None
        record = self.watcher.flush()
        self.note("flush", mark=label, flush_ms=record.get("flush_ms"), events_so_far=record.get("events_so_far"))

    def presize(self, path: Path | None = None, size: int = 65536) -> None:
        path = path or self.target
        fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_EXCL, 0o644)
        os.write(fd, b"x" * size)
        os.fsync(fd)
        os.close(fd)

    def settle_and_watch(self) -> None:
        time.sleep(self.args.settle)
        self.watcher = Watcher(Path(self.args.livewatch), self.dir)
        self.note("watch-ready", **{k: v for k, v in self.watcher.ready.items() if k != "type"})
        self.flush("post-start")

    def hold(self, label: str, seconds: float, fd: int | None = None, path: str | None = None) -> None:
        """Hold with the descriptor open: flush at +1 s, replay mid-hold, sample stat, flush at end."""
        time.sleep(1.0)
        self.flush(f"{label}+1s")
        self.sample(f"{label}+1s", fd, path)
        time.sleep(max(0.0, seconds / 2 - 1.0))
        self.replay_now(f"{label}-mid")
        time.sleep(max(0.0, seconds / 2))
        self.flush(f"{label}+{seconds:g}s")
        self.sample(f"{label}+{seconds:g}s", fd, path)

    def replay_now(self, label: str) -> None:
        assert self.watcher is not None
        # The global counter at stream start bounds the same window the live stream sees; the
        # device-time fence lags it because fseventsd writes its on-disk log lazily.
        cursor = int(self.watcher.ready["global_now"])
        result = replay(Path(self.args.probe), self.dir, cursor, self.args.replay_seconds)
        result["device_fence"] = int(self.watcher.ready["device_fence"])
        self.note("replay", mark=label, replay=result)

    def finish(self, extra: dict[str, Any] | None = None) -> dict[str, Any]:
        assert self.watcher is not None
        time.sleep(self.args.post)
        self.flush("post-close")
        self.sample("post-close")
        self.replay_now("after-close")
        summary = self.watcher.close()
        events = [r for r in self.watcher.records if r.get("type") == "event"]
        marks = [r for r in self.watcher.records if r.get("type") in ("mark", "flushed")]
        for event in events:
            event["names"] = flag_names(event["flags"])
        return {
            "op": self.op, "rep": self.rep, "trial_dir": str(self.dir.relative_to(self.args.root)),
            "watcher": {k: v for k, v in self.watcher.ready.items() if k != "type"},
            "events": events, "marks": marks, "log": self.log, "stats": self.stats,
            "watcher_summary": summary, "hold_seconds": self.args.hold, **(extra or {}),
        }


def run_op(args: argparse.Namespace, op: str, rep: int) -> dict[str, Any]:
    t = Trial(args, op, rep)
    hold = args.hold
    extra: dict[str, Any] = {}
    if op in ("append_fsync", "pwrite_inplace", "ftruncate_grow", "ftruncate_shrink", "prealloc",
              "fsync_only", "futimes", "fchmod", "fsetxattr", "clonefile_src", "fullfsync_append",
              "dup_close", "open_trunc"):
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        flags = os.O_RDWR | (os.O_TRUNC if op == "open_trunc" else 0)
        t.mark("open")
        fd = os.open(t.target, flags)
        t.mark("op-start")
        dup_fd = None
        if op == "append_fsync":
            os.lseek(fd, 0, os.SEEK_END)
            os.write(fd, b"a" * 16384)
            os.fsync(fd)
        elif op == "fullfsync_append":
            os.lseek(fd, 0, os.SEEK_END)
            os.write(fd, b"a" * 16384)
            fcntl.fcntl(fd, F_FULLFSYNC)
        elif op == "pwrite_inplace":
            os.pwrite(fd, b"p" * 4096, 8192)
            os.fsync(fd)
        elif op == "ftruncate_grow":
            os.ftruncate(fd, 131072)
            os.fsync(fd)
        elif op == "ftruncate_shrink":
            os.ftruncate(fd, 16384)
            os.fsync(fd)
        elif op == "prealloc":
            fst = struct.pack("IiqqQ", F_ALLOCATECONTIG | F_ALLOCATEALL, F_PEOFPOSMODE, 0, 1 << 20, 0)
            try:
                out = fcntl.fcntl(fd, F_PREALLOCATE, fst)
                extra["prealloc_bytesalloc"] = struct.unpack("IiqqQ", out)[4]
            except OSError:
                fst = struct.pack("IiqqQ", F_ALLOCATEALL, F_PEOFPOSMODE, 0, 1 << 20, 0)
                out = fcntl.fcntl(fd, F_PREALLOCATE, fst)
                extra["prealloc_bytesalloc"] = struct.unpack("IiqqQ", out)[4]
                extra["prealloc_contig_failed"] = True
        elif op == "fsync_only":
            os.fsync(fd)
        elif op == "futimes":
            os.utime(fd, ns=(1_600_000_000_000_000_000, 1_600_000_000_000_000_000))
        elif op == "fchmod":
            os.fchmod(fd, 0o640)
        elif op == "fsetxattr":
            fsetxattr = libc.fsetxattr
            fsetxattr.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint32, ctypes.c_int]
            value = b"review"
            if fsetxattr(fd, b"user.review", value, len(value), 0, 0) != 0:
                raise OSError(ctypes.get_errno(), "fsetxattr")
        elif op == "clonefile_src":
            os.lseek(fd, 0, os.SEEK_END)
            os.write(fd, b"c" * 4096)  # dirty the open descriptor first
            if libc.clonefile(bytes(t.target), bytes(t.dir / "clone"), 0) != 0:
                raise OSError(ctypes.get_errno(), "clonefile")
        elif op == "dup_close":
            dup_fd = os.dup(fd)
            os.lseek(fd, 0, os.SEEK_END)
            os.write(fd, b"d" * 16384)
            os.fsync(fd)
        elif op == "open_trunc":
            os.write(fd, b"t" * 4096)
            os.fsync(fd)
        t.mark("op-done")
        t.sample("op-done", fd)
        if op == "dup_close":
            t.mark("close-original-dup-alive")
            os.close(fd)
            t.hold("dup-alive", hold, dup_fd)
            t.mark("close-last")
            os.close(dup_fd)
        else:
            t.hold("open", hold, fd)
            t.mark("close")
            os.close(fd)
        t.mark("closed")
        return t.finish(extra)

    if op in ("mmap_fd_open", "mmap_fd_closed"):
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        t.mark("open")
        fd = os.open(t.target, os.O_RDWR)
        addr = libc.mmap(None, 65536, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
        if addr in (None, ctypes.c_void_p(-1).value):
            raise OSError(ctypes.get_errno(), "mmap")
        t.mark("mapped")
        if op == "mmap_fd_closed":
            os.close(fd)
            t.mark("fd-closed-before-write")
            fd = None
        t.mark("op-start")
        ctypes.memmove(addr + 4096, b"m" * 8192, 8192)
        t.sample("after-memory-write", fd)
        if libc.msync(addr, 65536, MS_SYNC) != 0:
            raise OSError(ctypes.get_errno(), "msync")
        t.mark("op-done")
        t.sample("op-done", fd)
        t.hold("mapped", hold, fd)
        if fd is not None:
            t.mark("close-fd-mapping-alive")
            os.close(fd)
            time.sleep(2.0)
            t.flush("fd-closed+2s")
            t.sample("fd-closed+2s")
        t.mark("munmap")
        libc.munmap(addr, 65536)
        t.mark("closed")
        return t.finish()

    if op == "sqlite_wal":
        t.settle_and_watch()
        db = t.dir / "target.db"
        t.mark("connect")
        conn = sqlite3.connect(str(db), isolation_level=None)
        conn.execute("PRAGMA journal_mode=WAL")
        extra["synchronous"] = conn.execute("PRAGMA synchronous").fetchone()[0]
        conn.execute("CREATE TABLE t(id INTEGER PRIMARY KEY, payload BLOB)")
        t.mark("op-start")
        for i in range(200):
            conn.execute("INSERT INTO t(payload) VALUES (?)", (b"w" * 512,))
        t.mark("inserts-done")
        names = ["target.db", "target.db-wal", "target.db-shm"]
        for name in names:
            t.sample(f"inserts-done:{name}", None, str(t.dir / name))
        t.hold("inserted", hold, None, str(db))
        for name in names:
            t.sample(f"inserted-hold:{name}", None, str(t.dir / name))
        t.mark("checkpoint")
        extra["checkpoint"] = conn.execute("PRAGMA wal_checkpoint(PASSIVE)").fetchone()
        t.mark("checkpoint-done")
        time.sleep(2.0)
        t.flush("checkpoint+2s")
        for name in names:
            t.sample(f"checkpoint+2s:{name}", None, str(t.dir / name))
        for i in range(50):
            conn.execute("INSERT INTO t(payload) VALUES (?)", (b"w" * 512,))
        t.mark("more-inserts-done")
        time.sleep(2.0)
        t.flush("more-inserts+2s")
        t.mark("close")
        conn.close()
        t.mark("closed")
        for name in names:
            t.sample(f"closed:{name}", None, str(t.dir / name))
        return t.finish(extra)

    if op == "rename_over_open":
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        t.mark("open")
        fd = os.open(t.target, os.O_RDWR)
        os.lseek(fd, 0, os.SEEK_END)
        os.write(fd, b"o" * 4096)
        os.fsync(fd)
        t.mark("op-start")
        newfd = os.open(t.dir / "target.new", os.O_RDWR | os.O_CREAT | os.O_EXCL, 0o644)
        os.write(newfd, b"n" * 1024)
        os.fsync(newfd)
        os.close(newfd)
        t.mark("new-written")
        os.rename(t.dir / "target.new", t.target)
        t.mark("renamed")
        avail_before = statvfs_avail(str(t.dir))
        os.lseek(fd, 0, os.SEEK_END)
        os.write(fd, b"g" * (1 << 20))
        os.fsync(fd)
        extra["avail_delta_after_orphan_growth"] = statvfs_avail(str(t.dir)) - avail_before
        t.mark("op-done")
        t.sample("op-done", fd)
        t.hold("orphaned", hold, fd)
        t.mark("close")
        os.close(fd)
        t.mark("closed")
        extra["avail_delta_after_close"] = statvfs_avail(str(t.dir)) - avail_before
        return t.finish(extra)

    if op == "unlink_open":
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        t.mark("open")
        fd = os.open(t.target, os.O_RDWR)
        os.lseek(fd, 0, os.SEEK_END)
        os.write(fd, b"u" * 4096)
        os.fsync(fd)
        t.mark("op-start")
        avail0 = statvfs_avail(str(t.dir))
        os.unlink(t.target)
        t.mark("unlinked")
        t.sample("unlinked", fd)
        os.lseek(fd, 0, os.SEEK_END)
        os.write(fd, b"g" * (1 << 20))
        os.fsync(fd)
        extra["avail_delta_after_orphan_growth"] = statvfs_avail(str(t.dir)) - avail0
        t.mark("op-done")
        t.sample("op-done", fd)
        t.hold("orphaned", hold, fd)
        t.mark("close")
        os.close(fd)
        t.mark("closed")
        time.sleep(0.5)
        extra["avail_delta_after_close"] = statvfs_avail(str(t.dir)) - avail0
        return t.finish(extra)

    if op == "second_writer_append":
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        t.mark("open")
        fd = os.open(t.target, os.O_RDWR)
        os.lseek(fd, 0, os.SEEK_END)
        os.write(fd, b"1" * 4096)
        os.fsync(fd)
        t.mark("op-start")
        code = ("import os,sys; fd=os.open(sys.argv[1], os.O_WRONLY|os.O_APPEND); os.write(fd, b'2'*8192); "
                "os.fsync(fd); os.close(fd)")
        subprocess.run([sys.executable, "-c", code, str(t.target)], check=True)
        t.mark("op-done")
        t.sample("op-done", fd)
        t.hold("second-closed-first-open", hold, fd)
        t.mark("close")
        os.close(fd)
        t.mark("closed")
        return t.finish()

    if op in ("child_sigkill", "child_exit"):
        t.presize()
        t.settle_and_watch()
        t.sample("before")
        code = ("import os,sys,time; fd=os.open(sys.argv[1], os.O_RDWR); os.lseek(fd,0,os.SEEK_END); "
                "os.write(fd, b'k'*16384); os.fsync(fd); print('written', flush=True); "
                + ("time.sleep(60)" if op == "child_sigkill" else "time.sleep(float(sys.argv[2])); os._exit(0)"))
        t.mark("op-start")
        child = subprocess.Popen([sys.executable, "-c", code, str(t.target), str(hold)], stdout=subprocess.PIPE, text=True)
        assert child.stdout is not None
        child.stdout.readline()
        t.mark("op-done")
        t.sample("op-done")
        if op == "child_sigkill":
            t.hold("child-open", hold)
            t.mark("close")
            child.kill()
            child.wait()
            t.mark("closed")
        else:
            t.hold("child-open", hold - 2)
            t.mark("close")
            child.wait()
            t.mark("closed")
        return t.finish({"child_returncode": child.returncode})

    raise ValueError(op)


ALL_OPS = [
    "append_fsync", "pwrite_inplace", "ftruncate_grow", "ftruncate_shrink", "mmap_fd_open",
    "mmap_fd_closed", "sqlite_wal", "rename_over_open", "unlink_open", "prealloc",
    "second_writer_append", "fsync_only", "fullfsync_append", "dup_close", "open_trunc", "futimes",
    "fchmod", "fsetxattr", "clonefile_src", "child_sigkill", "child_exit",
]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True)
    parser.add_argument("--livewatch", required=True)
    parser.add_argument("--probe", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--ops", default=",".join(ALL_OPS))
    parser.add_argument("--reps", type=int, default=3)
    parser.add_argument("--hold", type=float, default=10.0)
    parser.add_argument("--settle", type=float, default=5.0)
    parser.add_argument("--post", type=float, default=3.0)
    parser.add_argument("--replay-seconds", type=int, default=10)
    args = parser.parse_args()
    root = Path(args.root)
    if root.parts[1] != "Volumes":
        raise SystemExit("fixture root must be on the external scratch volume")
    root.mkdir(parents=True, exist_ok=True)
    ops = [op for op in args.ops.split(",") if op]
    with open(args.out, "a") as out:
        for rep in range(1, args.reps + 1):
            for op in ops:
                started = time.monotonic()
                try:
                    record = run_op(args, op, rep)
                except Exception as error:  # record the failure and continue
                    record = {"op": op, "rep": rep, "error": repr(error)}
                record["trial_wall_seconds"] = round(time.monotonic() - started, 3)
                record["host"] = {"load": os.getloadavg(), "time": time.time()}
                out.write(json.dumps(record) + "\n")
                out.flush()
                print(f"{op} r{rep}: {record.get('error', 'ok')} ({record['trial_wall_seconds']} s)", flush=True)


if __name__ == "__main__":
    main()
