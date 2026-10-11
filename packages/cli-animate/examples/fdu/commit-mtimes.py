"""Give each entry of a checkout the time of the last commit that changed it.

usage: python3 -I commit-mtimes.py GIT_DIR REV TREE

A fresh clone stamps every file with the moment of the clone, so a tree's age column
reads the same everywhere. This reads history from GIT_DIR, which a blobless clone
(`git clone --bare --filter=blob:none`) serves without the file contents, and walks back
from REV until every path in REV's tree has been dated. A path that arrived only through
a merge is dated by the merge that brought it onto the first-parent line. Files and
symlinks take their commit time; each directory then takes the newest time among its
entries, deepest first, as a directory whose entries were last added at that commit
would.
"""

import os
import subprocess
import sys


def git(git_dir: str, *args: str) -> bytes:
    return subprocess.run(
        ["git", "--git-dir", git_dir, *args], check=True, capture_output=True
    ).stdout


def last_commit_times(git_dir: str, rev: str, paths: set[str]) -> dict[str, int]:
    """Each path's newest commit time, from one walk of the history that stops early."""
    remaining = set(paths)
    times: dict[str, int] = {}
    log = subprocess.Popen(
        ["git", "--git-dir", git_dir, "log", "--format=%x01%ct", "--name-only"]
        + ["--first-parent", "-m", "--no-renames", "-z", rev],
        stdout=subprocess.PIPE,
    )
    assert log.stdout is not None
    when = 0
    pending = b""
    while remaining:
        chunk = log.stdout.read(1 << 20)
        if not chunk:
            break
        pending += chunk
        *fields, pending = pending.split(b"\0")
        for field in fields:
            for part in field.split(b"\n"):
                if part.startswith(b"\x01"):
                    when = int(part[1:])
                elif part:
                    path = part.decode("utf-8", "surrogateescape")
                    if path in remaining:
                        remaining.discard(path)
                        times[path] = when
    log.kill()
    log.wait()
    for path in remaining:
        stamp = git(git_dir, "log", "-1", "-m", "--first-parent", "--format=%ct", rev, "--", path)
        times[path] = int(stamp)
    return times


def main() -> None:
    git_dir, rev, tree = sys.argv[1:4]
    listed = git(git_dir, "ls-tree", "-r", "-z", "--name-only", rev)
    paths = set(listed.decode("utf-8", "surrogateescape").split("\0")) - {""}
    times = last_commit_times(git_dir, rev, paths)

    dirs: dict[str, int] = {}
    for path, stamp in times.items():
        full = os.path.join(tree, path)
        if os.path.lexists(full):
            os.utime(full, (stamp, stamp), follow_symlinks=False)
        parent = os.path.dirname(path)
        while parent:
            dirs[parent] = max(dirs.get(parent, 0), stamp)
            parent = os.path.dirname(parent)
    for path in sorted(dirs, key=lambda p: p.count("/"), reverse=True):
        full = os.path.join(tree, path)
        if os.path.isdir(full):
            os.utime(full, (dirs[path], dirs[path]))
    print(f"dated {len(times):,} paths and {len(dirs):,} directories", file=sys.stderr)


if __name__ == "__main__":
    main()
