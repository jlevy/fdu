#!/usr/bin/env python3
"""Build a ~230k-entry fixture shaped like agent/dev data, deterministically, so two copies
are identical (one to mark, one as the unmarked control).

  scale_build.py DEST [--scale F]

Layout under DEST:
  wide/                one directory of 20,000 small files
  deep-0..2/           three binary trees, depth 11, 4 files per dir (~20k entries each)
  target/              cargo-target-like: deps (10k files, 4-64 KiB), incremental (200x50), build (500x20)
  src/                 2,000 dirs x 30 files
  node_modules/        3,000 dirs x 15 files
File sizes are drawn from a seeded distribution mostly 1-8 KiB.
Prints the entry count and wall time.
"""
import os
import random
import sys
import time


def write(path, size):
    with open(path, "wb") as fh:
        fh.write(b"x" * size)


def main():
    dest = os.path.abspath(sys.argv[1])
    scale = 1.0
    if "--scale" in sys.argv:
        scale = float(sys.argv[sys.argv.index("--scale") + 1])
    rnd = random.Random(20260927)
    os.makedirs(dest)
    t0 = time.perf_counter()
    n = 0

    def sz():
        r = rnd.random()
        return rnd.randint(1, 8192) if r < 0.9 else rnd.randint(8192, 65536)

    # wide
    d = os.path.join(dest, "wide"); os.makedirs(d)
    for i in range(int(20000 * scale)):
        write(os.path.join(d, f"w{i:06d}.json"), sz()); n += 1

    # deep trees
    def deep(path, depth):
        nonlocal n
        for f in range(4):
            write(os.path.join(path, f"f{f}.rs"), sz()); n += 1
        if depth == 0:
            return
        for s in range(2):
            p = os.path.join(path, f"d{s}"); os.makedirs(p); n += 1
            deep(p, depth - 1)
    for t in range(3):
        p = os.path.join(dest, f"deep-{t}"); os.makedirs(p); n += 1
        deep(p, max(1, int(11 * (scale ** 0.25))))

    # target-like
    tg = os.path.join(dest, "target", "debug"); os.makedirs(tg); n += 2
    deps = os.path.join(tg, "deps"); os.makedirs(deps); n += 1
    for i in range(int(10000 * scale)):
        write(os.path.join(deps, f"lib{i:05d}-{rnd.getrandbits(32):08x}.rlib"), rnd.randint(4096, 65536)); n += 1
    inc = os.path.join(tg, "incremental"); os.makedirs(inc); n += 1
    for i in range(int(200 * scale)):
        p = os.path.join(inc, f"crate{i:03d}-{rnd.getrandbits(32):08x}"); os.makedirs(p); n += 1
        for f in range(50):
            write(os.path.join(p, f"s-{f:03d}.bin"), sz()); n += 1
    bld = os.path.join(tg, "build"); os.makedirs(bld); n += 1
    for i in range(int(500 * scale)):
        p = os.path.join(bld, f"build{i:04d}", "out"); os.makedirs(p); n += 2
        for f in range(20):
            write(os.path.join(p, f"o{f:02d}.o"), sz()); n += 1

    # src-like
    src = os.path.join(dest, "src"); os.makedirs(src); n += 1
    for i in range(int(2000 * scale)):
        p = os.path.join(src, f"m{i // 100:02d}", f"mod{i:04d}"); os.makedirs(p, exist_ok=True); n += 1
        for f in range(30):
            write(os.path.join(p, f"{f:02d}.py"), sz()); n += 1

    # node_modules-like
    nm = os.path.join(dest, "node_modules"); os.makedirs(nm); n += 1
    for i in range(int(3000 * scale)):
        p = os.path.join(nm, f"pkg-{i:04d}", "lib"); os.makedirs(p); n += 2
        for f in range(15):
            write(os.path.join(p, f"{f:02d}.js"), sz()); n += 1
    os.sync()
    print(f"built {dest}: ~{n} entries in {time.perf_counter() - t0:.1f} s")


if __name__ == "__main__":
    main()
