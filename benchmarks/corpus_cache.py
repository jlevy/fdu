"""Safe run-scoped reuse of independently verified benchmark corpora."""

from __future__ import annotations

import ctypes
import hashlib
import json
import os
import shutil
import stat
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, Mapping, Optional, Tuple

from benchmarks.corpus import (
    CORPUS_NAME,
    MANIFEST_NAME,
    CorpusError,
    cleanup_run_directory,
    create_corpus,
    refresh_copied_corpus,
    reserve_run_directory,
    verify_corpus,
)


BASE_CACHE_SCHEMA = "fdu-corpus-base-cache-v1"
MAX_BOUNDED_COPY_BYTES = 8 * 1024 * 1024 * 1024
_FICLONE = 0x40049409


@dataclass(frozen=True)
class _CopyStrategy:
    name: str
    clone_file: Optional[Callable[[Path, Path], None]]


@dataclass
class _CopyStats:
    cloned_files: int = 0
    copied_files: int = 0
    copied_bytes: int = 0
    directories: int = 0
    hardlinks: int = 0
    symlinks: int = 0


@dataclass(frozen=True)
class _BaseCorpus:
    cache_key: str
    manifest: Dict[str, Any]
    run_root: Path


class CorpusBasePool:
    """Own pristine baselines for one result run and materialize every trial from them."""

    def __init__(self, work_root: Path) -> None:
        candidate = Path(work_root)
        if candidate.is_symlink():
            raise CorpusError("corpus base-pool root may not be a symlink")
        try:
            self._work_root = candidate.resolve(strict=True)
        except OSError as error:
            raise CorpusError(f"cannot resolve corpus base-pool root: {error}") from error
        if not self._work_root.is_dir():
            raise CorpusError("corpus base-pool root is not a directory")
        probe_started = time.perf_counter_ns()
        self._strategy = _choose_copy_strategy(self._work_root)
        self._strategy_probe_ns = time.perf_counter_ns() - probe_started
        self._bases: Dict[Tuple[str, Optional[int], Optional[str]], _BaseCorpus] = {}
        self._closed = False

    def __enter__(self) -> CorpusBasePool:
        return self

    def __exit__(
        self,
        exception_type: Any,
        exception: Optional[BaseException],
        _traceback: Any,
    ) -> None:
        try:
            self.close()
        except CorpusError as cleanup_error:
            if exception_type is None:
                raise
            if exception is None:
                raise AssertionError("context manager received an exception type only")
            note = f"corpus base-pool cleanup also failed: {cleanup_error}"
            if hasattr(exception, "add_note"):
                exception.add_note(note)
            else:
                # PEP 678 notes arrived in 3.11 and this harness supports 3.9. Emulating
                # the attribute keeps the primary exception propagating while still
                # carrying the secondary failure, rather than losing one of two real
                # problems or replacing the error the caller expects.
                notes = list(getattr(exception, "__notes__", []))
                notes.append(note)
                exception.__notes__ = notes

    def materialize(
        self,
        run_root: Path,
        recipe_id: str,
        *,
        target_entries: Optional[int],
        seed: Optional[str],
    ) -> Tuple[Dict[str, Any], Dict[str, Any]]:
        """Populate an empty marked run from one pristine verified baseline."""
        if self._closed:
            raise CorpusError("corpus base pool is closed")
        destination = Path(run_root)
        corpus_destination = destination / CORPUS_NAME
        manifest_destination = destination / MANIFEST_NAME
        if (
            corpus_destination.exists()
            or corpus_destination.is_symlink()
            or manifest_destination.exists()
            or manifest_destination.is_symlink()
        ):
            raise CorpusError("corpus materialization destination is not empty")

        request = (recipe_id, target_entries, seed)
        base = self._bases.get(request)
        base_created = base is None
        generation_ns: Optional[int] = None
        if base is None:
            generation_started = time.perf_counter_ns()
            base_root = reserve_run_directory(self._work_root)
            try:
                base_manifest = create_corpus(
                    base_root,
                    recipe_id,
                    target_entries=target_entries,
                    seed=seed,
                )
            except Exception:
                cleanup_run_directory(base_root)
                raise
            generation_ns = time.perf_counter_ns() - generation_started
            base = _BaseCorpus(
                cache_key=_base_cache_key(base_manifest, self._strategy.name),
                manifest=base_manifest,
                run_root=base_root,
            )
            self._bases[request] = base

        source_verification_started = time.perf_counter_ns()
        verified_base = verify_corpus(base.run_root)
        source_verification_ns = (
            time.perf_counter_ns() - source_verification_started
        )
        if verified_base != base.manifest:
            raise CorpusError("corpus base manifest changed after creation")

        materialization_started = time.perf_counter_ns()
        stats = _copy_tree(
            base.run_root / CORPUS_NAME,
            corpus_destination,
            self._strategy,
        )
        shutil.copy2(base.run_root / MANIFEST_NAME, manifest_destination)
        manifest = refresh_copied_corpus(destination, base.manifest)
        materialization_ns = time.perf_counter_ns() - materialization_started
        evidence = {
            "base_cache_key": base.cache_key,
            "base_created": base_created,
            "base_generation_ns": generation_ns,
            "cloned_files": stats.cloned_files,
            "copied_bytes": stats.copied_bytes,
            "copied_files": stats.copied_files,
            "copy_strategy": self._strategy.name,
            "directories": stats.directories,
            "hardlinks": stats.hardlinks,
            "materialization_ns": materialization_ns,
            "source_verification_ns": source_verification_ns,
            "strategy_probe_ns": self._strategy_probe_ns,
            "symlinks": stats.symlinks,
        }
        return manifest, evidence

    def close(self) -> None:
        """Verify and remove every owned base, reporting cleanup failures together."""
        if self._closed:
            return
        self._closed = True
        errors = []
        for base in self._bases.values():
            try:
                verify_corpus(base.run_root)
            except CorpusError as error:
                errors.append(f"base verification failed: {error}")
            try:
                cleanup_run_directory(base.run_root)
            except CorpusError as error:
                errors.append(f"base cleanup failed: {error}")
        self._bases.clear()
        if errors:
            raise CorpusError("; ".join(errors))


def _base_cache_key(manifest: Mapping[str, Any], strategy: str) -> str:
    identity = {
        "capabilities": manifest["capabilities"],
        "copy_strategy": strategy,
        "generator_source_hash": manifest["generator_source_hash"],
        "generator_version": manifest["generator_version"],
        "manifest_schema": manifest["schema"],
        "recipe_hash": manifest["recipe_hash"],
        "schema": BASE_CACHE_SCHEMA,
    }
    encoded = json.dumps(
        identity, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _choose_copy_strategy(work_root: Path) -> _CopyStrategy:
    candidates = []
    if sys.platform == "darwin":
        clonefile = _darwin_clonefile()
        if clonefile is not None:
            candidates.append(_CopyStrategy("darwin-clonefile-v1", clonefile))
    if sys.platform.startswith("linux"):
        candidates.append(_CopyStrategy("linux-ficlone-v1", _linux_ficlone))
    for candidate in candidates:
        if _strategy_is_independent(candidate, work_root):
            return candidate
    return _CopyStrategy("bounded-copy-v1", None)


def _darwin_clonefile() -> Optional[Callable[[Path, Path], None]]:
    try:
        clonefile = ctypes.CDLL(None, use_errno=True).clonefile
    except (AttributeError, OSError):
        return None
    clonefile.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int]
    clonefile.restype = ctypes.c_int

    def clone(source: Path, destination: Path) -> None:
        if clonefile(os.fsencode(source), os.fsencode(destination), 0) != 0:
            errno_value = ctypes.get_errno()
            raise OSError(errno_value, os.strerror(errno_value), str(destination))

    return clone


def _linux_ficlone(source: Path, destination: Path) -> None:
    import fcntl

    with source.open("rb") as input_file, destination.open("xb") as output_file:
        fcntl.ioctl(output_file.fileno(), _FICLONE, input_file.fileno())


def _strategy_is_independent(strategy: _CopyStrategy, work_root: Path) -> bool:
    if strategy.clone_file is None:
        return False
    probe_root = Path(
        tempfile.mkdtemp(prefix=".fdu-copy-probe-", dir=str(work_root))
    )
    source = probe_root / "source"
    destination = probe_root / "destination"
    try:
        source.write_bytes(b"fdu-copy-on-write-probe")
        strategy.clone_file(source, destination)
        if not destination.is_file() or source.samefile(destination):
            return False
        destination.write_bytes(b"changed")
        return source.read_bytes() == b"fdu-copy-on-write-probe"
    except OSError:
        return False
    finally:
        shutil.rmtree(probe_root)


def _copy_tree(
    source_root: Path,
    destination_root: Path,
    strategy: _CopyStrategy,
) -> _CopyStats:
    if destination_root.exists() or destination_root.is_symlink():
        raise CorpusError(f"copy destination already exists: {destination_root}")
    if source_root.is_symlink() or not source_root.is_dir():
        raise CorpusError("corpus base root is missing or is a symlink")
    destination_root.mkdir()
    stats = _CopyStats(directories=1)
    hardlinks: Dict[Tuple[int, int], Path] = {}
    pending = [(source_root, destination_root)]
    directories = [(source_root, destination_root)]
    while pending:
        source_directory, destination_directory = pending.pop()
        try:
            entries = sorted(os.scandir(source_directory), key=lambda item: item.name)
        except OSError as error:
            raise CorpusError(f"cannot enumerate corpus base: {error}") from error
        child_directories = []
        for entry in entries:
            source = Path(entry.path)
            destination = destination_directory / entry.name
            try:
                metadata = source.lstat()
            except OSError as error:
                raise CorpusError(f"cannot stat corpus base entry: {error}") from error
            mode = metadata.st_mode
            if stat.S_ISDIR(mode):
                destination.mkdir()
                stats.directories += 1
                child_directories.append((source, destination))
                directories.append((source, destination))
            elif stat.S_ISLNK(mode):
                os.symlink(os.readlink(source), destination)
                stats.symlinks += 1
            elif stat.S_ISREG(mode):
                identity = (int(metadata.st_dev), int(metadata.st_ino))
                has_trustworthy_identity = identity != (0, 0)
                prior = (
                    hardlinks.get(identity)
                    if metadata.st_nlink > 1 and has_trustworthy_identity
                    else None
                )
                if prior is not None:
                    os.link(prior, destination)
                    stats.hardlinks += 1
                else:
                    _copy_regular(source, destination, metadata, strategy, stats)
                    if metadata.st_nlink > 1 and has_trustworthy_identity:
                        hardlinks[identity] = destination
            else:
                raise CorpusError(f"unsupported corpus base entry: {source.name}")
        pending.extend(reversed(child_directories))

    for source, destination in reversed(directories):
        shutil.copystat(source, destination, follow_symlinks=False)
    return stats


def _copy_regular(
    source: Path,
    destination: Path,
    metadata: os.stat_result,
    strategy: _CopyStrategy,
    stats: _CopyStats,
) -> None:
    if strategy.clone_file is not None:
        try:
            strategy.clone_file(source, destination)
        except OSError:
            if destination.exists() or destination.is_symlink():
                destination.unlink()
        else:
            shutil.copystat(source, destination, follow_symlinks=False)
            stats.cloned_files += 1
            return
    size = int(metadata.st_size)
    if stats.copied_bytes + size > MAX_BOUNDED_COPY_BYTES:
        raise CorpusError(
            "bounded corpus copy exceeds its 8 GiB logical-byte safety limit; "
            "use a clone/reflink-capable work filesystem"
        )
    shutil.copy2(source, destination, follow_symlinks=False)
    stats.copied_bytes += size
    stats.copied_files += 1
