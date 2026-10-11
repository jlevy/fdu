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
import stat
import subprocess
import sys
from pathlib import Path


def git(git_dir: str, *args: str) -> bytes:
    return subprocess.run(
        ["git", "--git-dir", git_dir, *args], check=True, capture_output=True
    ).stdout


def last_commit_times(git_dir: str, rev: str, paths: set[str]) -> dict[str, int]:
    """Each path's newest commit time, from one walk of the history that stops early."""
    remaining = set(paths)
    times: dict[str, int] = {}
    log = subprocess.Popen(
        ["git", "--git-dir", git_dir, "log", "--format=%x00%ct", "--name-only"]
        + ["--first-parent", "-m", "--no-renames", "-z", rev],
        stdout=subprocess.PIPE,
    )
    assert log.stdout is not None
    when = 0
    header = False
    first_path = False
    pending = b""
    try:
        while remaining:
            chunk = log.stdout.read(1 << 20)
            if not chunk:
                break
            pending += chunk
            *fields, pending = pending.split(b"\0")
            for field in fields:
                # An empty NUL field frames each commit: a Git path cannot be empty.
                if not field:
                    header = True
                elif header:
                    when = int(field)
                    header = False
                    first_path = True
                else:
                    # Git inserts one newline before the first path, not between paths.
                    if first_path:
                        field = field.removeprefix(b"\n")
                        first_path = False
                    path = field.decode("utf-8", "surrogateescape")
                    if path in remaining:
                        remaining.discard(path)
                        times[path] = when
    finally:
        log.stdout.close()
        if log.poll() is None:
            log.kill()
        log.wait()
    for path in remaining:
        stamp = git(git_dir, "log", "-1", "-m", "--first-parent", "--format=%ct", rev, "--", path)
        times[path] = int(stamp)
    return times


def destination(tree: Path, path: str, *, directory: bool = False) -> Path:
    relative = Path(path)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("path is outside TREE")
    full = tree / relative
    parent = tree
    for part in relative.parts[:-1]:
        parent /= part
        mode = parent.lstat().st_mode
        if stat.S_ISLNK(mode):
            raise ValueError(f"symlink directory ancestor {str(parent.relative_to(tree))!r}")
        if not stat.S_ISDIR(mode):
            raise ValueError(f"ancestor is not a directory: {str(parent.relative_to(tree))!r}")
    if not full.parent.resolve(strict=True).is_relative_to(tree):
        raise ValueError("resolved parent is outside TREE")
    mode = full.lstat().st_mode
    if directory:
        if stat.S_ISLNK(mode):
            raise ValueError("directory is a symlink")
        if not stat.S_ISDIR(mode):
            raise ValueError("not a directory")
    return full


def main() -> int:
    git_dir, rev, tree_arg = sys.argv[1:4]
    try:
        tree = Path(tree_arg).resolve(strict=True)
        if not tree.is_dir():
            raise ValueError("not a directory")
    except (OSError, ValueError) as error:
        print(f"TREE must be an existing directory: {tree_arg!r}: {error}", file=sys.stderr)
        return 1
    listed = git(git_dir, "ls-tree", "-r", "-z", "--name-only", rev)
    paths = set(listed.decode("utf-8", "surrogateescape").split("\0")) - {""}
    times = last_commit_times(git_dir, rev, paths)

    dirs: dict[str, int] = {}
    updated_paths = skipped_paths = 0
    for path, stamp in times.items():
        try:
            full = destination(tree, path)
            os.utime(full, (stamp, stamp), follow_symlinks=False)
            updated_paths += 1
        except (OSError, ValueError) as error:
            print(f"skipped path {path!r}: {error}", file=sys.stderr)
            skipped_paths += 1
        parent = os.path.dirname(path)
        while parent:
            dirs[parent] = max(dirs.get(parent, 0), stamp)
            parent = os.path.dirname(parent)
    updated_dirs = skipped_dirs = 0
    for path in sorted(dirs, key=lambda p: (-p.count("/"), p)):
        try:
            full = destination(tree, path, directory=True)
            os.utime(full, (dirs[path], dirs[path]), follow_symlinks=False)
            updated_dirs += 1
        except (OSError, ValueError) as error:
            print(f"skipped directory {path!r}: {error}", file=sys.stderr)
            skipped_dirs += 1
    print(
        f"dated {updated_paths:,} paths and {updated_dirs:,} directories; "
        f"skipped {skipped_paths:,} paths and {skipped_dirs:,} directories",
        file=sys.stderr,
    )
    return int(bool(skipped_paths or skipped_dirs))


if __name__ == "__main__":
    sys.exit(main())
