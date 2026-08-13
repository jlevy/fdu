"""Path-redacted, mutation-detecting fingerprints for an external reference tree.

A reference tree is a real directory on the operator's machine — a checkout with a
large ``node_modules``, a source tree, a home directory. We never write to it, and we
never record a path from it. What we do record is enough to prove two things:

1. The subject under measurement observed exactly the tree the oracle observed.
   This is the same ``fdu-index-record-v1`` engine digest the generated-corpus
   harness uses, so a probe's ``summary.engine_digest`` is directly comparable.
2. The tree did not change between the baseline and a later comparison. Timings
   taken against different trees are not comparable, and a harness that quietly
   averages them is worse than no harness at all.

Redaction is structural, not cosmetic: the emitted fingerprint contains counts,
byte totals, a depth histogram, an extension histogram, and digests. It contains no
absolute path, no entry name, and no directory name. The tree is identified by
``root_id``, the SHA-256 of its absolute path, plus whatever label the operator
chose to give it.
"""

from __future__ import annotations

import hashlib
import os
import stat
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from benchmarks.corpus import (
    ENGINE_DIGEST_ALGORITHM,
    CorpusError,
    _engine_record_bytes,
    _SemanticAccumulator,
)

FINGERPRINT_SCHEMA = "fdu-reference-tree-v3"

#: Extensions are structural, not private, but an unbounded histogram would leak the
#: shape of a private tree one rare suffix at a time. Keep the head, count the tail.
_EXTENSION_HISTOGRAM_LIMIT = 24

#: A depth histogram is capped so a pathological tree cannot produce an unbounded
#: document. Anything deeper is folded into the final bucket.
_MAX_DEPTH_BUCKET = 32


class ReferenceTreeError(RuntimeError):
    """A reference tree could not be observed, or changed while it was observed."""


def fingerprint(root: Path, *, label: str) -> Dict[str, Any]:
    """Observe ``root`` exactly once and return its redacted fingerprint.

    The walk is a single non-following pass. Every entry is stat'd without following
    symlinks, which is what the engine does, so the digest is comparable to the
    engine's own. Anything unreadable is an error: a partial observation cannot
    prove a tree is unchanged.
    """
    absolute = root.resolve(strict=True)
    if not absolute.is_dir() or absolute.is_symlink():
        raise ReferenceTreeError("reference tree root must be a real directory")

    engine = _SemanticAccumulator(algorithm=ENGINE_DIGEST_ALGORITHM)
    engine.add_bytes(_engine_record_bytes(".", "directory", None))

    counts = {"directories": 1, "files": 0, "other": 0, "symlinks": 0, "total": 1}
    apparent_bytes = 0
    allocated_bytes_total = 0
    newest_file_mtime_ns: Optional[int] = None
    linked_files: Dict[Tuple[int, int], Tuple[int, int, int]] = {}
    depths: Dict[int, int] = {0: 1}
    extensions: Dict[str, int] = {}
    max_depth = 0
    unreadable: List[str] = []

    for relative, depth, metadata in _walk(absolute, unreadable):
        mode = metadata.st_mode
        if stat.S_ISDIR(mode):
            kind = "directory"
            counts["directories"] += 1
        elif stat.S_ISREG(mode):
            kind = "file"
            counts["files"] += 1
            apparent = int(metadata.st_size)
            allocated = _allocated_bytes(metadata)
            apparent_bytes += apparent
            allocated_bytes_total += allocated
            mtime_ns = int(metadata.st_mtime_ns)
            newest_file_mtime_ns = (
                mtime_ns
                if newest_file_mtime_ns is None
                else max(newest_file_mtime_ns, mtime_ns)
            )
            if int(metadata.st_nlink) > 1:
                key = (int(metadata.st_dev), int(metadata.st_ino))
                occurrences, recorded_apparent, recorded_allocated = linked_files.get(
                    key, (0, apparent, allocated)
                )
                linked_files[key] = (
                    occurrences + 1,
                    recorded_apparent,
                    recorded_allocated,
                )
            extensions[_extension(relative)] = extensions.get(_extension(relative), 0) + 1
        elif stat.S_ISLNK(mode):
            kind = "symlink"
            counts["symlinks"] += 1
        else:
            kind = "other"
            counts["other"] += 1
        counts["total"] += 1
        bucket = min(depth, _MAX_DEPTH_BUCKET)
        depths[bucket] = depths.get(bucket, 0) + 1
        max_depth = max(max_depth, depth)
        engine.add_bytes(_engine_record_bytes(relative, kind, metadata))

    if unreadable:
        raise ReferenceTreeError(
            f"{len(unreadable)} entries could not be observed; a partial observation "
            "cannot prove the tree is unchanged"
        )

    return {
        "counts": counts,
        "engine_digest": engine.finish(),
        "engine_digest_algorithm": ENGINE_DIGEST_ALGORITHM,
        "engine_digest_components": engine.components(),
        "extensions": _bounded_extensions(extensions),
        "hardlinks": _hardlink_summary(linked_files),
        "label": label,
        "max_depth": max_depth,
        "root_id": root_id(absolute),
        "schema": FINGERPRINT_SCHEMA,
        "sizes": {
            "allocated_bytes": allocated_bytes_total,
            "apparent_bytes": apparent_bytes,
        },
        "newest_file_mtime_ns": newest_file_mtime_ns,
        "depth_histogram": {str(key): depths[key] for key in sorted(depths)},
    }


def _allocated_bytes(metadata: Any) -> int:
    """Match the engine's honest fallback when allocated size is unavailable."""
    if os.name == "posix" and hasattr(metadata, "st_blocks"):
        return int(metadata.st_blocks) * 512
    return int(metadata.st_size)


def root_id(root: Path) -> str:
    """Identify a tree without disclosing where it is."""
    return hashlib.sha256(os.fsencode(Path(root).resolve(strict=False))).hexdigest()


def compare(current: Dict[str, Any], baseline: Dict[str, Any]) -> List[str]:
    """Return the reasons ``current`` and ``baseline`` describe different trees.

    An empty list means the two observations are of the same tree in the same state,
    and measurements taken against them may be compared.
    """
    reasons: List[str] = []
    if current.get("schema") != baseline.get("schema"):
        reasons.append(
            "reference tree fingerprint schema differs from the baseline schema"
        )
    if current.get("root_id") != baseline.get("root_id"):
        reasons.append("reference tree root differs from the baseline root")
    if current.get("engine_digest") != baseline.get("engine_digest"):
        reasons.append(
            "reference tree contents changed since the baseline "
            f"({_shape(baseline)} -> {_shape(current)})"
        )
    return reasons


def probe_agrees(fingerprint_document: Dict[str, Any], summary: Any) -> Optional[str]:
    """Return why a probe summary disagrees with the oracle, or ``None`` if it does not.

    The probe walks the tree itself and reports what it found. If that disagrees with
    an independent Python walk, either the tree moved underneath the trial or the
    engine is wrong; in both cases the timing is not evidence of anything.
    """
    if not isinstance(summary, dict):
        return "probe emitted no summary to check against the oracle"
    counts = fingerprint_document["counts"]
    expected = {
        "dirs": counts["directories"],
        "entries": counts["total"],
        "files": counts["files"],
        "other": counts["other"],
        "symlinks": counts["symlinks"],
        "apparent_bytes": fingerprint_document["sizes"]["apparent_bytes"],
        "allocated_bytes": fingerprint_document["sizes"]["allocated_bytes"],
        "newest_file_mtime_ns": fingerprint_document["newest_file_mtime_ns"],
        "engine_digest": fingerprint_document["engine_digest"],
    }
    for field, want in expected.items():
        got = summary.get(field)
        if got != want:
            return f"probe {field}={got!r} disagrees with oracle {want!r}"
    return None


def _walk(root: Path, unreadable: List[str]):
    """Yield ``(relative, depth, metadata)`` for every descendant, name-sorted."""
    pending: List[Tuple[Path, str, int]] = [(root, "", 0)]
    while pending:
        directory, prefix, depth = pending.pop()
        try:
            with os.scandir(directory) as handle:
                entries = sorted(handle, key=lambda entry: entry.name)
        except OSError:
            unreadable.append(prefix or ".")
            continue
        children: List[Tuple[Path, str, int]] = []
        for entry in entries:
            relative = entry.name if not prefix else f"{prefix}/{entry.name}"
            try:
                metadata = entry.stat(follow_symlinks=False)
            except OSError:
                unreadable.append(relative)
                continue
            yield relative, depth + 1, metadata
            if stat.S_ISDIR(metadata.st_mode):
                children.append((Path(entry.path), relative, depth + 1))
        pending.extend(reversed(children))


def _extension(relative: str) -> str:
    name = relative.rsplit("/", 1)[-1]
    stem, separator, suffix = name.rpartition(".")
    if not separator or not stem or not suffix or len(suffix) > 16:
        return ""
    return suffix.lower()


def _bounded_extensions(extensions: Dict[str, int]) -> Dict[str, Any]:
    ordered = sorted(extensions.items(), key=lambda item: (-item[1], item[0]))
    head = ordered[:_EXTENSION_HISTOGRAM_LIMIT]
    tail = ordered[_EXTENSION_HISTOGRAM_LIMIT:]
    return {
        "distinct": len(ordered),
        "top": {name: count for name, count in head},
        "remaining_files": sum(count for _name, count in tail),
    }


def _hardlink_summary(
    linked_files: Dict[Tuple[int, int], Tuple[int, int, int]],
) -> Dict[str, int]:
    """Describe in-tree hard-link duplication without retaining or exposing names."""
    groups = [entry for entry in linked_files.values() if entry[0] > 1]
    return {
        "groups": len(groups),
        "linked_file_entries": sum(entry[0] for entry in groups),
        "duplicate_file_entries": sum(entry[0] - 1 for entry in groups),
        "duplicate_apparent_bytes": sum(
            (entry[0] - 1) * entry[1] for entry in groups
        ),
        "duplicate_allocated_bytes": sum(
            (entry[0] - 1) * entry[2] for entry in groups
        ),
    }


def _shape(document: Dict[str, Any]) -> str:
    counts = document.get("counts", {})
    sizes = document.get("sizes", {})
    return (
        f"{counts.get('total')} entries / {sizes.get('apparent_bytes')} bytes"
    )


__all__ = [
    "FINGERPRINT_SCHEMA",
    "CorpusError",
    "ReferenceTreeError",
    "compare",
    "fingerprint",
    "probe_agrees",
    "root_id",
]
