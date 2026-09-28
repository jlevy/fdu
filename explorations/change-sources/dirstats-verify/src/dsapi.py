"""In-process ctypes bindings for APFS dir-stats probes (verification tooling).

gen(path)   -> dict: gencount, extflags, privatesize, cloneid, refcnt, objtype, datalen, dataalloc, alloc, rsrcalloc
get(path)   -> dict: gen, desc, phys (fsctl 0xC1104A71, zeroed struct)  -- raises OSError on failure
mark(path, w1=1, w0=0) -> dict of the returned struct (what apfs.util -M does when w1=1)
apfs_util_M(path), apfs_util_S(path) -> subprocess wrappers, to cross-check
"""
import ctypes
import ctypes.util
import os
import re
import struct
import subprocess
import time

_libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)
_libc.getattrlist.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_ulong]
_libc.getattrlist.restype = ctypes.c_int
_libc.fsctl.argtypes = [ctypes.c_char_p, ctypes.c_ulong, ctypes.c_void_p, ctypes.c_uint]
_libc.fsctl.restype = ctypes.c_int

APFS_UTIL = "/System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util"
DS_FSCTL = 0xC1104A71
DS_STRUCT = 272

ATTR_BIT_MAP_COUNT = 5
ATTR_CMN_RETURNED_ATTRS = 0x80000000
ATTR_CMN_OBJTYPE = 0x8
ATTR_FILE_ALLOCSIZE = 0x4
ATTR_FILE_DATALENGTH = 0x200
ATTR_FILE_DATAALLOCSIZE = 0x400
ATTR_FILE_RSRCALLOCSIZE = 0x2000
ATTR_CMNEXT_PRIVATESIZE = 0x8
ATTR_CMNEXT_CLONEID = 0x100
ATTR_CMNEXT_EXT_FLAGS = 0x200
ATTR_CMNEXT_RECURSIVE_GENCOUNT = 0x400
ATTR_CMNEXT_CLONE_REFCNT = 0x1000
FSOPT_NOFOLLOW = 0x1
FSOPT_ATTR_CMN_EXTENDED = 0x20

_FILEATTR = ATTR_FILE_ALLOCSIZE | ATTR_FILE_DATALENGTH | ATTR_FILE_DATAALLOCSIZE | ATTR_FILE_RSRCALLOCSIZE
_FORKATTR = ATTR_CMNEXT_PRIVATESIZE | ATTR_CMNEXT_CLONEID | ATTR_CMNEXT_EXT_FLAGS | ATTR_CMNEXT_RECURSIVE_GENCOUNT | ATTR_CMNEXT_CLONE_REFCNT
_ATTRLIST = struct.pack("<HHIIIII", ATTR_BIT_MAP_COUNT, 0, ATTR_CMN_RETURNED_ATTRS | ATTR_CMN_OBJTYPE, 0, 0, _FILEATTR, _FORKATTR)


def gen(path):
    """getattrlist with FSOPT_ATTR_CMN_EXTENDED; returns the CMNEXT attributes plus file sizes."""
    al = ctypes.create_string_buffer(_ATTRLIST, len(_ATTRLIST))
    buf = ctypes.create_string_buffer(512)
    rc = _libc.getattrlist(os.fsencode(path), al, buf, 512, FSOPT_ATTR_CMN_EXTENDED | FSOPT_NOFOLLOW)
    if rc != 0:
        e = ctypes.get_errno()
        raise OSError(e, os.strerror(e), path)
    raw = buf.raw
    off = 4
    c, v, d, f, k = struct.unpack_from("<IIIII", raw, off); off += 20
    r = {"objtype": None, "alloc": None, "datalen": None, "dataalloc": None, "rsrcalloc": None,
         "privatesize": None, "cloneid": None, "extflags": None, "gencount": None, "refcnt": None, "ret_fork": k}
    if c & ATTR_CMN_OBJTYPE:
        r["objtype"] = struct.unpack_from("<I", raw, off)[0]; off += 4
    for bit, key in ((ATTR_FILE_ALLOCSIZE, "alloc"), (ATTR_FILE_DATALENGTH, "datalen"), (ATTR_FILE_DATAALLOCSIZE, "dataalloc"), (ATTR_FILE_RSRCALLOCSIZE, "rsrcalloc")):
        if f & bit:
            r[key] = struct.unpack_from("<Q", raw, off)[0]; off += 8
    if k & ATTR_CMNEXT_PRIVATESIZE:
        r["privatesize"] = struct.unpack_from("<q", raw, off)[0]; off += 8
    if k & ATTR_CMNEXT_CLONEID:
        r["cloneid"] = struct.unpack_from("<Q", raw, off)[0]; off += 8
    if k & ATTR_CMNEXT_EXT_FLAGS:
        r["extflags"] = struct.unpack_from("<Q", raw, off)[0]; off += 8
    if k & ATTR_CMNEXT_RECURSIVE_GENCOUNT:
        r["gencount"] = struct.unpack_from("<Q", raw, off)[0]; off += 8
    if k & ATTR_CMNEXT_CLONE_REFCNT:
        r["refcnt"] = struct.unpack_from("<I", raw, off)[0]; off += 4
    return r


def gencount(path):
    return gen(path)["gencount"]


def _fsctl(path, buf):
    rc = _libc.fsctl(os.fsencode(path), DS_FSCTL, buf, 0)
    if rc != 0:
        e = ctypes.get_errno()
        raise OSError(e, os.strerror(e), path)


def get(path, raw=False):
    """fsctl GET_DIR_STATS_EXT with a zeroed struct. Returns gen/desc/phys and the call time in µs."""
    buf = ctypes.create_string_buffer(DS_STRUCT)
    t0 = time.perf_counter_ns()
    _fsctl(path, buf)
    dt = (time.perf_counter_ns() - t0) / 1000
    g, d, p = struct.unpack_from("<QQQ", buf.raw, 0x30)
    out = {"gen": g, "desc": d, "phys": p, "us": dt}
    if raw:
        out["words"] = {hex(i): struct.unpack_from("<Q", buf.raw, i)[0] for i in range(0, DS_STRUCT, 8) if struct.unpack_from("<Q", buf.raw, i)[0]}
    return out


def mark(path, w1=1, w0=0):
    """The apfs.util -M call: zeroed 272-byte struct, u32 at +4 = w1 (1), same fsctl. Returns the struct's nonzero words."""
    buf = ctypes.create_string_buffer(DS_STRUCT)
    struct.pack_into("<II", buf, 0, w0, w1)
    t0 = time.perf_counter_ns()
    _fsctl(path, buf)
    dt = (time.perf_counter_ns() - t0) / 1000
    g, d, p = struct.unpack_from("<QQQ", buf.raw, 0x30)
    return {"gen": g, "desc": d, "phys": p, "us": dt,
            "words": {hex(i): struct.unpack_from("<Q", buf.raw, i)[0] for i in range(0, DS_STRUCT, 8) if struct.unpack_from("<Q", buf.raw, i)[0]}}


def apfs_util_M(path):
    r = subprocess.run([APFS_UTIL, "-M", path], capture_output=True, text=True)
    return {"rc": r.returncode, "out": r.stdout.strip(), "err": r.stderr.strip()}


def apfs_util_S(path):
    r = subprocess.run([APFS_UTIL, "-S", path], capture_output=True, text=True)
    out = {"rc": r.returncode, "err": r.stderr.strip()}
    for key, pat in (("desc", r"descendants:\s*(\d+)"), ("phys", r"physical size:\s*(\d+)"), ("gen", r"gen-count:\s*(\d+)")):
        m = re.search(pat, r.stdout)
        out[key] = int(m.group(1)) if m else None
    return out
