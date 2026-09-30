"""Write a file whole, or leave the old one in place.

A reader of a file written here sees the old contents or the new ones, never a torn file
under the final name, even after a crash or a container restart mid-write. The data goes
to a temporary beside the target, is flushed and synced, and is then renamed over the
target; on any failure the temporary is removed and the target is untouched. This is
"Write Every File Whole" in docs/project/architecture/fdu-design-principles.md, and
scripts/check-atomic-writes.mjs fails on a raw write anywhere outside a helper.

Each function mirrors the call it replaces, so converting one changes nothing but
atomicity: text defaults to the same encoding and newline translation as ``open``, a
new file gets the permissions ``open`` would give it, a replaced file keeps its own, and
a symbolic link is written through rather than replaced. Mode ``"x"`` still refuses an
existing target, and publishes with a hard link so a racing writer cannot be clobbered.

An append-only file, such as a JSONL log, is the one exception to whole writes: it grows a
record at a time, so a crash can cut its last record short. ``complete_lines`` is how its
readers drop that torn record instead of failing on it or, worse, reading it as whole.

explorations/benchmarks/atomic_write.py is a byte-identical copy for the benchmark
harness, a separate project that cannot import this one; the check keeps the two equal.
"""

from __future__ import annotations

import contextlib
import errno
import os
import secrets
import stat
from collections.abc import Iterator
from typing import IO, Any

StrPath = str | os.PathLike[str]

_MODES = frozenset({"w", "wb", "wt", "x", "xb", "xt"})
_CREATE_ATTEMPTS = 64


def write_text_atomic(
    path: StrPath,
    data: str,
    encoding: str | None = None,
    errors: str | None = None,
    newline: str | None = None,
) -> None:
    """Replace ``path`` with ``data``, as ``Path.write_text`` would, but whole."""
    with open_atomic(path, "w", encoding=encoding, errors=errors, newline=newline) as output:
        output.write(data)


def write_bytes_atomic(path: StrPath, data: bytes) -> None:
    """Replace ``path`` with ``data``, as ``Path.write_bytes`` would, but whole."""
    with open_atomic(path, "wb") as output:
        output.write(data)


def complete_lines(path: StrPath, encoding: str | None = None) -> list[str]:
    """The newline-terminated lines of an append-only file, without a torn last record.

    Its writer ends every record with a newline, so a final line without one is a record
    a crash cut short, and it is dropped. Lines are returned without their newlines.
    """
    with open(path, encoding=encoding) as source:
        return source.read().split("\n")[:-1]


@contextlib.contextmanager
def open_atomic(
    path: StrPath,
    mode: str = "w",
    *,
    encoding: str | None = None,
    errors: str | None = None,
    newline: str | None = None,
) -> Iterator[IO[Any]]:
    """Yield a file that becomes ``path`` when the block completes without an exception.

    ``mode`` is one of ``w``, ``wb``, ``x`` and ``xb`` (``t`` allowed). The handle is a
    real file, so a child process may write to its descriptor. An exception inside the
    block removes the temporary and leaves ``path`` as it was.
    """
    if mode not in _MODES:
        raise ValueError(f"open_atomic writes a whole new file, not mode {mode!r}")
    exclusive = mode.startswith("x")
    if exclusive and os.path.lexists(path):
        raise FileExistsError(errno.EEXIST, os.strerror(errno.EEXIST), os.fspath(path))
    target = os.path.realpath(path) if os.path.islink(path) else os.fspath(path)

    descriptor, temporary = _create_temporary(target)
    try:
        if not exclusive:
            _keep_permissions(descriptor, target)
        binary = "b" in mode
        handle = os.fdopen(
            descriptor,
            "wb" if binary else "w",
            encoding=None if binary else encoding,
            errors=None if binary else errors,
            newline=None if binary else newline,
        )
    except BaseException:
        os.close(descriptor)
        _discard(temporary)
        raise

    try:
        with handle:
            yield handle
            handle.flush()
            os.fsync(handle.fileno())
        if exclusive:
            # A hard link fails when the name exists, so a writer that won a race keeps
            # its file; the temporary is removed either way.
            try:
                os.link(temporary, target)
            finally:
                _discard(temporary)
        else:
            os.replace(temporary, target)
    except BaseException:
        _discard(temporary)
        raise


def _create_temporary(target: str) -> tuple[int, str]:
    """Create a new temporary beside ``target``, as ``open`` would create the target."""
    directory, name = os.path.split(target)
    for _ in range(_CREATE_ATTEMPTS):
        temporary = os.path.join(directory, f".{name}.tmp.{os.getpid()}.{secrets.token_hex(8)}")
        try:
            # 0o666 under the umask is what ``open`` gives a new file.
            flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0)
            return os.open(temporary, flags, 0o666), temporary
        except FileExistsError:
            continue
    raise FileExistsError(errno.EEXIST, "could not reserve a unique temporary", target)


def _keep_permissions(descriptor: int, target: str) -> None:
    """Give the temporary the permissions of the file it replaces, if there is one."""
    if not hasattr(os, "fchmod"):
        return
    try:
        existing = os.stat(target)
    except FileNotFoundError:
        return
    os.fchmod(descriptor, stat.S_IMODE(existing.st_mode))


def _discard(temporary: str) -> None:
    with contextlib.suppress(FileNotFoundError):
        os.unlink(temporary)
