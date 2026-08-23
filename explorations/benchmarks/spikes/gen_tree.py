#!/usr/bin/env python3
"""Generate a heterogeneous benchmark tree: node_modules-heavy JS projects,
src trees, sparse git packs, build artifacts, symlinks. Deterministic."""
import os, random, sys

ROOT = sys.argv[1]
rng = random.Random(42)

dirs_made = files_made = links_made = 0

CODE_EXT = [".js", ".ts", ".tsx", ".json", ".md", ".map", ".mjs", ".cjs", ".d.ts"]
SRC_EXT = [".rs", ".py", ".go", ".c", ".h", ".toml", ".yaml", ".md", ".txt"]
ASSET_EXT = [".png", ".svg", ".woff2", ".ico"]

def mkfile(path, size):
    global files_made
    fd = os.open(path, os.O_CREAT | os.O_WRONLY, 0o644)
    if size > 0:
        if size <= 256:
            os.write(fd, b"x" * size)
        else:
            os.truncate(fd, size)  # sparse: realistic st_size, tiny disk use
    os.close(fd)
    files_made += 1

def mkdirp(path):
    global dirs_made
    os.makedirs(path, exist_ok=True)
    dirs_made += 1

def small():  return rng.choice([0, 12, 64, 180, 240, 900, 1400, 3200, 5100])
def medium(): return rng.randint(8_000, 220_000)
def big():    return rng.randint(1_000_000, 60_000_000)

def make_package(base, depth, budget):
    """One npm-style package; returns entries created."""
    made = 0
    mkdirp(base); made += 1
    mkfile(os.path.join(base, "package.json"), small()); made += 1
    mkfile(os.path.join(base, "README.md"), small()); made += 1
    n_lib = rng.randint(4, 60)
    libdir = os.path.join(base, rng.choice(["lib", "dist", "src", "build"]))
    mkdirp(libdir); made += 1
    for i in range(n_lib):
        mkfile(os.path.join(libdir, f"m{i}{rng.choice(CODE_EXT)}"), small())
        made += 1
    if rng.random() < 0.25:
        extra = os.path.join(libdir, "esm")
        mkdirp(extra); made += 1
        for i in range(rng.randint(2, 20)):
            mkfile(os.path.join(extra, f"e{i}{rng.choice(CODE_EXT)}"), small())
            made += 1
    if depth < 9 and budget > 400 and rng.random() < 0.5:
        nm = os.path.join(base, "node_modules")
        mkdirp(nm); made += 1
        for i in range(rng.randint(1, 6)):
            if budget - made < 200: break
            made += make_package(os.path.join(nm, f"dep-{depth}-{i}"), depth + 1, (budget - made) // 2)
    return made

def make_src_tree(base, depth):
    made = 0
    mkdirp(base); made += 1
    for i in range(rng.randint(5, 30)):
        mkfile(os.path.join(base, f"f{i}{rng.choice(SRC_EXT)}"), rng.choice([small(), small(), medium()]))
        made += 1
    if depth < 6:
        for i in range(rng.randint(0, 4)):
            made += make_src_tree(os.path.join(base, f"mod{i}"), depth + 1)
    return made

total_target = int(sys.argv[2]) if len(sys.argv) > 2 else 450_000
made_total = 0
proj = 0
while made_total < total_target:
    p = os.path.join(ROOT, f"project-{proj:02d}")
    mkdirp(p); made_total += 1
    # src
    made_total += make_src_tree(os.path.join(p, "src"), 0)
    # .git-ish with packs
    gd = os.path.join(p, ".git", "objects", "pack")
    os.makedirs(gd); dirs_made += 3; made_total += 3
    for i in range(rng.randint(1, 4)):
        mkfile(os.path.join(gd, f"pack-{i:02d}.pack"), big()); made_total += 1
    # assets
    ad = os.path.join(p, "assets")
    mkdirp(ad); made_total += 1
    for i in range(rng.randint(10, 80)):
        mkfile(os.path.join(ad, f"a{i}{rng.choice(ASSET_EXT)}"), medium()); made_total += 1
    # node_modules: the bulk
    nm = os.path.join(p, "node_modules")
    mkdirp(nm); made_total += 1
    budget = min(total_target - made_total, rng.randint(15_000, 45_000))
    k = 0
    while budget > 300:
        made = make_package(os.path.join(nm, f"pkg-{k:03d}"), 0, budget)
        made_total += made
        budget -= made
        k += 1
    # a few symlinks
    for i in range(rng.randint(3, 12)):
        try:
            os.symlink("../package.json", os.path.join(nm, f"link-{i}"))
            links_made += 1; made_total += 1
        except FileExistsError:
            pass
    proj += 1

print(f"entries~{made_total} dirs={dirs_made} files={files_made} links={links_made} projects={proj}")
