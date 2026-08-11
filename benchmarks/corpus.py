"""Deterministic corpus generation and independent filesystem verification."""

from __future__ import annotations

import copy
import hashlib
import json
import math
import os
import shutil
import stat
import struct
import tempfile
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Dict, Iterator, List, Mapping, Optional, Sequence, Tuple


RECIPE_SCHEMA = "fdu-corpus-recipes-v1"
MANIFEST_SCHEMA = "fdu-observed-corpus-v2"
MARKER_SCHEMA = "fdu-perf-run-v1"
GENERATOR_VERSION = 2
MARKER_NAME = ".fdu-perf-run-v1"
MANIFEST_NAME = "observed-corpus.json"
CORPUS_NAME = "corpus"
RUN_PREFIX = "fdu-perf-"
SEMANTIC_DIGEST_ALGORITHM = "sha256-multiset-v1"
ENGINE_DIGEST_ALGORITHM = "fdu-index-record-v1/sha256-multiset-v1"
ROLLUP_DIGEST_ALGORITHM = "fdu-rollup-record-v1/sha256-multiset-v1"
MAX_TARGET_ENTRIES = 10_000_000
DETAIL_ENTRY_LIMIT = 512
MAX_JSON_BYTES = 8 * 1024 * 1024
BASE_MTIME_SECONDS = 1_700_000_000
PROJECT_ROOT = Path(__file__).resolve().parent.parent

# Windows directory-enumeration metadata omits file identity and may be stale. The
# independent oracle needs fresh identity to distinguish hardlinks from unrelated files.
_DIRENTRY_METADATA_IS_AUTHORITATIVE = os.name != "nt"

_RECIPE_KEYS = {
    "default_target_entries",
    "fanout",
    "max_depth",
    "metadata_profile",
    "mutation_locality",
    "optional_links",
    "scale_points",
    "seed",
    "topology",
    "transitions",
}
_TOPOLOGIES = {"balanced", "contract", "deep", "wide"}
_METADATA_PROFILES = {"contract", "empty", "mixed", "partial", "standard"}
_OPTIONAL_LINKS = {"hardlink", "symlink"}
_TRANSITIONS = {"mixed-1pct", "modify-1pct", "one-change"}


class CorpusError(ValueError):
    """Raised when a corpus operation cannot preserve its safety contract."""


@dataclass(frozen=True)
class Recipe:
    """A validated, effective corpus recipe."""

    recipe_id: str
    seed: str
    target_entries: int
    topology: str
    fanout: int
    max_depth: int
    metadata_profile: str
    optional_links: Tuple[str, ...]
    transitions: Tuple[str, ...]
    mutation_locality: Optional[str]
    scale_points: Tuple[int, ...]

    def as_mapping(self) -> Dict[str, Any]:
        """Return the canonical recipe representation used for hashing."""
        return {
            "fanout": self.fanout,
            "max_depth": self.max_depth,
            "metadata_profile": self.metadata_profile,
            "mutation_locality": self.mutation_locality,
            "optional_links": list(self.optional_links),
            "recipe_id": self.recipe_id,
            "scale_points": list(self.scale_points),
            "seed": self.seed,
            "target_entries": self.target_entries,
            "topology": self.topology,
            "transitions": list(self.transitions),
        }


class _SemanticAccumulator:
    """Build a stable multiset digest without retaining or sorting every record."""

    def __init__(
        self,
        components: Optional[Mapping[str, Any]] = None,
        *,
        algorithm: str = SEMANTIC_DIGEST_ALGORITHM,
    ) -> None:
        self._algorithm = algorithm
        if components is None:
            self._count = 0
            self._xor = bytearray(32)
            self._sum = 0
            return
        count = components.get("count")
        xor_value = components.get("xor")
        sum_value = components.get("sum")
        if (
            not isinstance(count, int)
            or isinstance(count, bool)
            or count < 0
            or not isinstance(xor_value, str)
            or not isinstance(sum_value, str)
        ):
            raise CorpusError("semantic digest components are malformed")
        try:
            xor_bytes = bytes.fromhex(xor_value)
            parsed_sum = int(sum_value, 16)
        except ValueError as error:
            raise CorpusError("semantic digest components are malformed") from error
        if len(xor_bytes) != 32 or len(sum_value) != 64 or parsed_sum >= 1 << 256:
            raise CorpusError("semantic digest components are malformed")
        self._count = count
        self._xor = bytearray(xor_bytes)
        self._sum = parsed_sum

    def add(self, record: Mapping[str, Any]) -> None:
        self.add_bytes(_canonical_json(record))

    def add_bytes(self, record: bytes) -> None:
        """Add one already normalized binary record."""
        leaf = hashlib.sha256(record).digest()
        self._add_leaf(leaf)

    def _add_leaf(self, leaf: bytes) -> None:
        for index, value in enumerate(leaf):
            self._xor[index] ^= value
        self._sum = (self._sum + int.from_bytes(leaf, "big")) % (1 << 256)
        self._count += 1

    def remove(self, record: Mapping[str, Any]) -> None:
        if self._count == 0:
            raise CorpusError("cannot remove a record from an empty semantic digest")
        leaf = hashlib.sha256(_canonical_json(record)).digest()
        for index, value in enumerate(leaf):
            self._xor[index] ^= value
        self._sum = (self._sum - int.from_bytes(leaf, "big")) % (1 << 256)
        self._count -= 1

    def components(self) -> Dict[str, Any]:
        return {
            "count": self._count,
            "sum": f"{self._sum:064x}",
            "xor": bytes(self._xor).hex(),
        }

    def finish(self) -> str:
        digest = hashlib.sha256()
        digest.update(self._algorithm.encode("ascii"))
        digest.update(b"\0")
        digest.update(self._count.to_bytes(8, "big"))
        digest.update(bytes(self._xor))
        digest.update(self._sum.to_bytes(32, "big"))
        return digest.hexdigest()


class _RollupAccumulator:
    """Independently derive every directory reducer from observed filesystem data."""

    def __init__(self) -> None:
        self._directories: Dict[str, Dict[str, Any]] = {".": _empty_rollup()}

    def add(self, relative: str, kind: str, metadata: os.stat_result) -> None:
        if kind == "directory":
            self._directories.setdefault(relative, _empty_rollup())
            for ancestor in _ancestor_directories(relative):
                self._directories.setdefault(ancestor, _empty_rollup())["dirs"] += 1
            return
        if kind != "file":
            return

        size = int(metadata.st_size)
        allocated = (
            int(metadata.st_blocks) * 512
            if os.name == "posix" and hasattr(metadata, "st_blocks")
            else size
        )
        extension = _rollup_extension(relative)
        for ancestor in _ancestor_directories(relative):
            rollup = self._directories.setdefault(ancestor, _empty_rollup())
            rollup["files"] += 1
            rollup["bytes"] += size
            rollup["allocated"] += allocated
            newest = rollup["newest_mtime_ns"]
            rollup["newest_mtime_ns"] = (
                int(metadata.st_mtime_ns)
                if newest is None
                else max(newest, int(metadata.st_mtime_ns))
            )
            if extension is not None:
                tally = rollup["by_ext"].setdefault(
                    extension, {"bytes": 0, "files": 0}
                )
                tally["files"] += 1
                tally["bytes"] += size

    def summary(self) -> Dict[str, Any]:
        digest = _SemanticAccumulator(algorithm=ROLLUP_DIGEST_ALGORITHM)
        for path, rollup in sorted(self._directories.items()):
            digest.add_bytes(_rollup_record_bytes(path, rollup))
        return {
            "rollup_digest": digest.finish(),
            "rollup_digest_components": digest.components(),
        }


@dataclass
class _TransitionLedger:
    semantic: _SemanticAccumulator
    operations: List[Dict[str, Any]]
    changed_paths: set[str]


class _ManifestBuilder:
    def __init__(self, capture_records: bool) -> None:
        self._capture_records = capture_records
        self._records: List[Dict[str, Any]] = []
        self._semantic = _SemanticAccumulator()
        self._counts = {
            "directories": 0,
            "files": 0,
            "other": 0,
            "symlinks": 0,
        }
        self._child_counts: Dict[str, int] = {}
        self._extensions: Dict[str, Dict[str, int]] = {}
        self._hardlink_groups: set[str] = set()
        self._entry_apparent_bytes = 0
        self._unique_apparent_bytes = 0
        self._max_depth = 0

    def add(self, record: Dict[str, Any]) -> None:
        relative_path = _require_relative_record_path(record.get("path"))
        kind = record.get("kind")
        count_key = {
            "directory": "directories",
            "file": "files",
            "other": "other",
            "symlink": "symlinks",
        }.get(kind)
        if count_key is None:
            raise CorpusError(f"unknown record kind: {kind!r}")

        self._counts[count_key] += 1
        self._max_depth = max(self._max_depth, _path_depth(relative_path))
        if relative_path != ".":
            parent = str(PurePosixPath(relative_path).parent)
            self._child_counts[parent] = self._child_counts.get(parent, 0) + 1

        if kind == "file":
            size = record.get("size")
            if not isinstance(size, int) or isinstance(size, bool) or size < 0:
                raise CorpusError(f"invalid file size for {relative_path!r}")
            self._entry_apparent_bytes += size
            hardlink_to = record.get("hardlink_to")
            if hardlink_to is None:
                self._unique_apparent_bytes += size
            else:
                canonical = _require_relative_record_path(hardlink_to)
                self._hardlink_groups.add(canonical)
            extension = _extension_for(relative_path)
            extension_totals = self._extensions.setdefault(
                extension, {"apparent_bytes": 0, "count": 0}
            )
            extension_totals["apparent_bytes"] += size
            extension_totals["count"] += 1

        normalized = dict(record)
        self._semantic.add(normalized)
        if self._capture_records:
            self._records.append(normalized)

    def summary(self) -> Dict[str, Any]:
        counts = dict(self._counts)
        counts["hardlink_groups"] = len(self._hardlink_groups)
        counts["total"] = sum(self._counts.values())
        summary: Dict[str, Any] = {
            "counts": counts,
            "extensions": dict(sorted(self._extensions.items())),
            "semantic_components": self._semantic.components(),
            "semantic_digest": self._semantic.finish(),
            "sizes": {
                "entry_apparent_bytes": self._entry_apparent_bytes,
                "unique_apparent_bytes": self._unique_apparent_bytes,
            },
            "topology": {
                "max_depth": self._max_depth,
                "max_directory_fanout": max(self._child_counts.values(), default=0),
            },
        }
        if self._capture_records:
            summary["records"] = sorted(
                self._records, key=lambda record: record["path"]
            )
        return summary


def reserve_run_directory(base_directory: Path) -> Path:
    """Reserve a unique marked run directory below an explicit base directory."""
    base = Path(base_directory)
    try:
        base.mkdir(parents=True, exist_ok=True)
        resolved_base = base.resolve(strict=True)
    except OSError as error:
        raise CorpusError(f"cannot prepare run-directory base: {error}") from error
    if not resolved_base.is_dir():
        raise CorpusError(f"run-directory base is not a directory: {base}")

    run_root = Path(tempfile.mkdtemp(prefix=RUN_PREFIX, dir=str(resolved_base)))
    resolved_run_root = run_root.resolve(strict=True)
    marker = {
        "resolved_path": str(resolved_run_root),
        "run_id": resolved_run_root.name,
        "schema": MARKER_SCHEMA,
    }
    try:
        _atomic_write_json(resolved_run_root / MARKER_NAME, marker)
    except Exception:
        resolved_run_root.rmdir()
        raise
    return resolved_run_root


def cleanup_run_directory(run_root: Path) -> None:
    """Remove exactly one validated benchmark run directory."""
    resolved = _validate_run_root(run_root)
    with _operation_lock(resolved):
        try:
            shutil.rmtree(resolved)
        except OSError as error:
            raise CorpusError(
                f"cannot remove validated run directory: {error}"
            ) from error


def create_corpus(
    run_root: Path,
    recipe_id: str,
    *,
    target_entries: Optional[int] = None,
    seed: Optional[str] = None,
) -> Dict[str, Any]:
    """Create and independently verify one corpus in a reserved run directory."""
    resolved_run_root = _validate_run_root(run_root)
    with _operation_lock(resolved_run_root):
        return _create_corpus_unlocked(
            resolved_run_root,
            recipe_id,
            target_entries=target_entries,
            seed=seed,
        )


def _create_corpus_unlocked(
    run_root: Path,
    recipe_id: str,
    *,
    target_entries: Optional[int],
    seed: Optional[str],
) -> Dict[str, Any]:
    resolved_run_root = _validate_run_root(run_root)
    recipe = load_recipe(recipe_id, target_entries=target_entries, seed=seed)
    corpus_root = resolved_run_root / CORPUS_NAME
    if corpus_root.exists() or corpus_root.is_symlink():
        raise CorpusError(f"corpus target already exists: {corpus_root}")
    corpus_root.mkdir()

    capture_records = recipe.target_entries + 3 <= DETAIL_ENTRY_LIMIT
    expected = _ManifestBuilder(capture_records=capture_records)
    capabilities = _initial_capabilities(recipe)
    if recipe.topology == "contract":
        _generate_contract(corpus_root, recipe, expected, capabilities)
    else:
        _generate_parametric(corpus_root, recipe, expected, capabilities)
    _set_directory_mtimes(corpus_root, recipe.seed)

    expected_summary = expected.summary()
    observed_summary, observed_capabilities = _observe_corpus(
        corpus_root, capture_records=capture_records
    )
    _assert_expected_matches_observed(expected_summary, observed_summary)
    capabilities.update(observed_capabilities)

    manifest: Dict[str, Any] = {
        "capabilities": capabilities,
        "counts": observed_summary["counts"],
        "extensions": observed_summary["extensions"],
        "generator_source_hash": _generator_source_hash(),
        "generator_version": GENERATOR_VERSION,
        "oracle": {
            "creation_matches_observation": True,
            "digest_algorithm": SEMANTIC_DIGEST_ALGORITHM,
            "engine_digest": observed_summary["engine_digest"],
            "engine_digest_algorithm": ENGINE_DIGEST_ALGORITHM,
            "engine_digest_components": observed_summary[
                "engine_digest_components"
            ],
            "rollup_digest": observed_summary["rollup_digest"],
            "rollup_digest_algorithm": ROLLUP_DIGEST_ALGORITHM,
            "rollup_digest_components": observed_summary[
                "rollup_digest_components"
            ],
            "root_inclusive": True,
            "semantic_components": observed_summary["semantic_components"],
        },
        "recipe": recipe.as_mapping(),
        "recipe_hash": _sha256_json(recipe.as_mapping()),
        "recipe_id": recipe.recipe_id,
        "schema": MANIFEST_SCHEMA,
        "semantic_digest": observed_summary["semantic_digest"],
        "sizes": observed_summary["sizes"],
        "state": {"id": "baseline", "transition_index": 0},
        "topology": observed_summary["topology"],
    }
    if "records" in observed_summary:
        manifest["records"] = observed_summary["records"]
    manifest["manifest_hash"] = _manifest_hash(manifest)
    _atomic_write_json(resolved_run_root / MANIFEST_NAME, manifest)
    return manifest


def verify_corpus(run_root: Path) -> Dict[str, Any]:
    """Verify the current corpus against its stored observed manifest."""
    resolved_run_root = _validate_run_root(run_root)
    with _operation_lock(resolved_run_root):
        return _verify_corpus_unlocked(resolved_run_root)


def refresh_copied_corpus(
    run_root: Path, source_manifest: Mapping[str, Any]
) -> Dict[str, Any]:
    """Validate a copied corpus and bind its manifest to fresh filesystem identity."""
    resolved_run_root = _validate_run_root(run_root)
    with _operation_lock(resolved_run_root):
        copied = _load_manifest(resolved_run_root / MANIFEST_NAME)
        if copied != source_manifest:
            raise CorpusError("copied corpus manifest differs from its verified base")
        corpus_root = resolved_run_root / CORPUS_NAME
        if corpus_root.is_symlink() or not corpus_root.is_dir():
            raise CorpusError("the copied corpus is missing or is a symlink")
        capture_records = "records" in copied
        observed, observed_capabilities = _observe_corpus(
            corpus_root, capture_records=capture_records
        )
        expected_summary: Dict[str, Any] = {
            "counts": copied["counts"],
            "extensions": copied["extensions"],
            "semantic_components": copied["oracle"]["semantic_components"],
            "semantic_digest": copied["semantic_digest"],
            "sizes": copied["sizes"],
            "topology": copied["topology"],
        }
        if capture_records:
            expected_summary["records"] = copied["records"]
        _assert_expected_matches_observed(expected_summary, observed)

        refreshed = copy.deepcopy(copied)
        refreshed["capabilities"].update(observed_capabilities)
        refreshed["sizes"] = observed["sizes"]
        refreshed["oracle"]["engine_digest"] = observed["engine_digest"]
        refreshed["oracle"]["engine_digest_components"] = observed[
            "engine_digest_components"
        ]
        refreshed["oracle"]["rollup_digest"] = observed["rollup_digest"]
        refreshed["oracle"]["rollup_digest_components"] = observed[
            "rollup_digest_components"
        ]
        refreshed["manifest_hash"] = _manifest_hash(refreshed)
        _atomic_write_json(resolved_run_root / MANIFEST_NAME, refreshed)
        return _verify_corpus_unlocked(resolved_run_root)


def _verify_corpus_unlocked(run_root: Path) -> Dict[str, Any]:
    resolved_run_root = _validate_run_root(run_root)
    manifest = _load_manifest(resolved_run_root / MANIFEST_NAME)
    corpus_root = resolved_run_root / CORPUS_NAME
    if corpus_root.is_symlink() or not corpus_root.is_dir():
        raise CorpusError("the run corpus is missing or is a symlink")
    capture_records = "records" in manifest
    observed, _ = _observe_corpus(corpus_root, capture_records=capture_records)
    for field in (
        "counts",
        "extensions",
        "semantic_digest",
        "sizes",
        "topology",
    ):
        if manifest.get(field) != observed.get(field):
            raise CorpusError(f"corpus verification failed for {field}")
    if capture_records and manifest.get("records") != observed.get("records"):
        raise CorpusError("corpus verification failed for records")
    if manifest.get("oracle", {}).get("semantic_components") != observed.get(
        "semantic_components"
    ):
        raise CorpusError("corpus verification failed for semantic components")
    if manifest.get("oracle", {}).get("engine_digest") != observed.get(
        "engine_digest"
    ):
        raise CorpusError("corpus verification failed for engine digest")
    if manifest.get("oracle", {}).get("engine_digest_components") != observed.get(
        "engine_digest_components"
    ):
        raise CorpusError("corpus verification failed for engine digest components")
    if manifest.get("oracle", {}).get("rollup_digest") != observed.get(
        "rollup_digest"
    ):
        raise CorpusError("corpus verification failed for roll-up digest")
    if manifest.get("oracle", {}).get("rollup_digest_components") != observed.get(
        "rollup_digest_components"
    ):
        raise CorpusError("corpus verification failed for roll-up digest components")
    return manifest


def apply_transition(run_root: Path, transition_id: str) -> Dict[str, Any]:
    """Apply the next declared churn transition and persist its exact oracle."""
    resolved_run_root = _validate_run_root(run_root)
    with _operation_lock(resolved_run_root):
        return _apply_transition_unlocked(resolved_run_root, transition_id)


def _apply_transition_unlocked(
    run_root: Path, transition_id: str
) -> Dict[str, Any]:
    resolved_run_root = _validate_run_root(run_root)
    manifest = _verify_corpus_unlocked(resolved_run_root)
    recipe, transition_index = _transition_recipe_and_index(manifest, transition_id)
    corpus_root = resolved_run_root / CORPUS_NAME
    selected = _selected_mutation_candidates(corpus_root, recipe, transition_id)
    ledger = _TransitionLedger(
        semantic=_SemanticAccumulator(
            manifest["oracle"].get("semantic_components")
        ),
        operations=[],
        changed_paths=set(),
    )
    _apply_selected_transition(
        corpus_root,
        recipe,
        transition_id,
        transition_index + 1,
        selected,
        ledger,
    )
    capture_records = "records" in manifest
    observed, observed_capabilities = _observe_corpus(
        corpus_root, capture_records=capture_records
    )
    _validate_transition_postcondition(manifest, observed, ledger.semantic)
    updated = _updated_transition_manifest(
        manifest,
        observed,
        observed_capabilities,
        transition_id,
        transition_index + 1,
        ledger,
        capture_records=capture_records,
    )
    _atomic_write_json(resolved_run_root / MANIFEST_NAME, updated)
    return updated


def _transition_recipe_and_index(
    manifest: Mapping[str, Any], transition_id: str
) -> Tuple[Recipe, int]:
    recipe_mapping = manifest.get("recipe")
    if not isinstance(recipe_mapping, dict):
        raise CorpusError("manifest recipe is malformed")
    recipe = load_recipe(
        str(manifest.get("recipe_id")),
        target_entries=recipe_mapping.get("target_entries"),
        seed=recipe_mapping.get("seed"),
    )
    if recipe.as_mapping() != recipe_mapping:
        raise CorpusError("manifest recipe no longer matches the committed recipe")
    transition_index = manifest.get("state", {}).get("transition_index")
    if not isinstance(transition_index, int) or isinstance(transition_index, bool):
        raise CorpusError("manifest transition state is malformed")
    if transition_index >= len(recipe.transitions):
        raise CorpusError("the recipe has no remaining transitions")
    expected_transition = recipe.transitions[transition_index]
    if transition_id != expected_transition:
        raise CorpusError(
            f"transition order is invalid: expected {expected_transition!r}, "
            f"received {transition_id!r}"
        )
    return recipe, transition_index


def _selected_mutation_candidates(
    corpus_root: Path, recipe: Recipe, transition_id: str
) -> List[Tuple[str, int, int]]:
    candidates = _mutation_candidates(corpus_root, recipe, transition_id)
    requested_count = _transition_change_count(transition_id, len(candidates))
    if len(candidates) < requested_count:
        raise CorpusError(
            f"transition {transition_id!r} needs {requested_count} independent files, "
            f"but the corpus has {len(candidates)}"
        )
    return candidates[:requested_count]


def _apply_selected_transition(
    corpus_root: Path,
    recipe: Recipe,
    transition_id: str,
    transition_index: int,
    selected: Sequence[Tuple[str, int, int]],
    ledger: _TransitionLedger,
) -> None:
    if transition_id in {"one-change", "modify-1pct"}:
        for operation_index, candidate in enumerate(selected):
            _apply_file_modification(
                corpus_root,
                recipe,
                transition_index,
                operation_index,
                candidate,
                ledger,
            )
        return

    touched_directories = {
        str(PurePosixPath(candidate[0]).parent)
        for operation_index, candidate in enumerate(selected)
        if operation_index % 3 in {1, 2}
    }
    original_directory_mtimes = {
        relative: (
            corpus_root if relative == "." else corpus_root / relative
        ).stat().st_mtime_ns
        for relative in touched_directories
    }
    _apply_mixed_file_operations(
        corpus_root, recipe, transition_index, selected, ledger
    )
    _update_touched_directory_records(
        corpus_root,
        recipe,
        transition_index,
        touched_directories,
        original_directory_mtimes,
        ledger,
    )


def _apply_file_modification(
    corpus_root: Path,
    recipe: Recipe,
    transition_index: int,
    operation_index: int,
    candidate: Tuple[str, int, int],
    ledger: _TransitionLedger,
) -> None:
    relative, size, old_mtime_ns = candidate
    new_mtime_ns = _mutation_mtime_ns(
        recipe.seed, transition_index, relative, operation_index
    )
    _rewrite_file_preserving_size(
        corpus_root / relative, new_mtime_ns, recipe.seed
    )
    ledger.semantic.remove(_file_record(relative, size, old_mtime_ns))
    ledger.semantic.add(_file_record(relative, size, new_mtime_ns))
    ledger.operations.append({"kind": "modify", "path": relative})
    ledger.changed_paths.add(relative)


def _apply_mixed_file_operations(
    corpus_root: Path,
    recipe: Recipe,
    transition_index: int,
    selected: Sequence[Tuple[str, int, int]],
    ledger: _TransitionLedger,
) -> None:
    for operation_index, candidate in enumerate(selected):
        relative, size, old_mtime_ns = candidate
        action = operation_index % 3
        if action == 0:
            _apply_file_modification(
                corpus_root,
                recipe,
                transition_index,
                operation_index,
                candidate,
                ledger,
            )
            continue

        prefix = "added" if action == 1 else "renamed"
        new_relative = _mutation_destination(
            relative,
            transition_index=transition_index,
            operation_index=operation_index,
            seed=recipe.seed,
            prefix=prefix,
        )
        destination = corpus_root / new_relative
        if destination.exists() or destination.is_symlink():
            raise CorpusError(f"mutation destination already exists: {new_relative}")
        new_mtime_ns = _mutation_mtime_ns(
            recipe.seed, transition_index, new_relative, operation_index
        )
        ledger.semantic.remove(_file_record(relative, size, old_mtime_ns))
        if action == 1:
            (corpus_root / relative).unlink()
            _create_file(corpus_root, new_relative, size, f"{recipe.seed}:mutation")
            ledger.operations.append(
                {"added": new_relative, "kind": "replace", "removed": relative}
            )
        else:
            os.replace(corpus_root / relative, destination)
            ledger.operations.append(
                {"from": relative, "kind": "rename", "to": new_relative}
            )
        _set_path_times_ns(destination, new_mtime_ns, new_mtime_ns)
        ledger.semantic.add(_file_record(new_relative, size, new_mtime_ns))
        ledger.changed_paths.update({relative, new_relative})


def _update_touched_directory_records(
    corpus_root: Path,
    recipe: Recipe,
    transition_index: int,
    touched_directories: set[str],
    original_mtimes: Mapping[str, int],
    ledger: _TransitionLedger,
) -> None:
    for directory_index, relative in enumerate(sorted(touched_directories)):
        directory_path = corpus_root if relative == "." else corpus_root / relative
        old_mtime_ns = original_mtimes[relative]
        new_mtime_ns = _mutation_mtime_ns(
            recipe.seed,
            transition_index,
            f"directory:{relative}",
            directory_index,
        )
        ledger.semantic.remove(
            {"kind": "directory", "mtime_ns": old_mtime_ns, "path": relative}
        )
        _set_path_times_ns(directory_path, new_mtime_ns, new_mtime_ns)
        ledger.semantic.add(
            {"kind": "directory", "mtime_ns": new_mtime_ns, "path": relative}
        )
        ledger.changed_paths.add(relative)


def _validate_transition_postcondition(
    manifest: Mapping[str, Any],
    observed: Mapping[str, Any],
    semantic: _SemanticAccumulator,
) -> None:
    if semantic.finish() != observed["semantic_digest"]:
        raise CorpusError("transition ledger disagrees with observed semantic digest")
    if semantic.components() != observed["semantic_components"]:
        raise CorpusError("transition ledger disagrees with observed digest components")
    for field in ("counts", "extensions", "topology"):
        if manifest.get(field) != observed.get(field):
            raise CorpusError(f"transition unexpectedly changed {field}")
    for size_field in ("entry_apparent_bytes", "unique_apparent_bytes"):
        previous_size = manifest.get("sizes", {}).get(size_field)
        if previous_size != observed["sizes"].get(size_field):
            raise CorpusError(f"transition unexpectedly changed sizes.{size_field}")


def _updated_transition_manifest(
    manifest: Mapping[str, Any],
    observed: Mapping[str, Any],
    observed_capabilities: Mapping[str, Any],
    transition_id: str,
    transition_index: int,
    ledger: _TransitionLedger,
    *,
    capture_records: bool,
) -> Dict[str, Any]:
    history = manifest.get("transition_history", [])
    if not isinstance(history, list):
        raise CorpusError("manifest transition history is malformed")
    transition_record = {
        "changed_paths": sorted(ledger.changed_paths),
        "id": transition_id,
        "index": transition_index,
        "operations": ledger.operations,
        "previous_manifest_hash": manifest["manifest_hash"],
    }
    updated = dict(manifest)
    updated.update(
        {
            "counts": observed["counts"],
            "extensions": observed["extensions"],
            "semantic_digest": observed["semantic_digest"],
            "sizes": observed["sizes"],
            "state": {"id": transition_id, "transition_index": transition_index},
            "topology": observed["topology"],
            "transition_history": history + [transition_record],
        }
    )
    updated_capabilities = dict(manifest.get("capabilities", {}))
    updated_capabilities.update(observed_capabilities)
    updated["capabilities"] = updated_capabilities
    updated_oracle = dict(manifest.get("oracle", {}))
    updated_oracle.update(
        {
            "engine_digest": observed["engine_digest"],
            "engine_digest_components": observed["engine_digest_components"],
            "rollup_digest": observed["rollup_digest"],
            "rollup_digest_components": observed["rollup_digest_components"],
            "semantic_components": observed["semantic_components"],
            "transition_matches_observation": True,
        }
    )
    updated["oracle"] = updated_oracle
    if capture_records:
        updated["records"] = observed["records"]
    updated.pop("manifest_hash", None)
    updated["manifest_hash"] = _manifest_hash(updated)
    return updated


def load_recipe(
    recipe_id: str,
    *,
    target_entries: Optional[int] = None,
    seed: Optional[str] = None,
) -> Recipe:
    """Load one committed recipe and apply validated scalar overrides."""
    document = _load_json(Path(__file__).with_name("corpora.json"))
    if not isinstance(document, dict) or set(document) != {"recipes", "schema"}:
        raise CorpusError("corpora.json has unknown or missing top-level fields")
    if document.get("schema") != RECIPE_SCHEMA:
        raise CorpusError(
            f"unsupported corpus recipe schema: {document.get('schema')!r}"
        )
    recipes = document.get("recipes")
    if not isinstance(recipes, dict) or recipe_id not in recipes:
        available = ", ".join(sorted(recipes)) if isinstance(recipes, dict) else "none"
        raise CorpusError(
            f"unknown recipe {recipe_id!r}; available recipes: {available}"
        )
    raw = recipes[recipe_id]
    if not isinstance(raw, dict):
        raise CorpusError(f"recipe {recipe_id!r} is not an object")
    unknown = set(raw) - _RECIPE_KEYS
    missing = _RECIPE_KEYS - {"mutation_locality"} - set(raw)
    if unknown or missing:
        raise CorpusError(
            f"recipe {recipe_id!r} has unknown fields {sorted(unknown)} "
            f"or missing fields {sorted(missing)}"
        )

    effective_target = (
        _require_int("target_entries", target_entries, minimum=1)
        if target_entries is not None
        else _require_int(
            "default_target_entries", raw.get("default_target_entries"), 1
        )
    )
    if effective_target > MAX_TARGET_ENTRIES:
        raise CorpusError(
            f"target_entries exceeds the safety limit of {MAX_TARGET_ENTRIES:,}"
        )
    if (
        recipe_id == "contract"
        and effective_target != raw.get("default_target_entries")
    ):
        raise CorpusError("the exact contract recipe does not accept a target override")

    topology = _require_choice("topology", raw.get("topology"), _TOPOLOGIES)
    metadata_profile = _require_choice(
        "metadata_profile", raw.get("metadata_profile"), _METADATA_PROFILES
    )
    optional_links = _require_string_sequence(
        "optional_links", raw.get("optional_links"), _OPTIONAL_LINKS
    )
    transitions = _require_string_sequence(
        "transitions", raw.get("transitions"), _TRANSITIONS
    )
    scale_points_raw = raw.get("scale_points")
    if not isinstance(scale_points_raw, list) or not scale_points_raw:
        raise CorpusError("scale_points must be a non-empty array")
    scale_points = tuple(
        _require_int("scale_point", value, minimum=1) for value in scale_points_raw
    )
    raw_seed = raw.get("seed") if seed is None else seed
    if not isinstance(raw_seed, str) or not raw_seed or len(raw_seed) > 200:
        raise CorpusError("seed must be a non-empty string of at most 200 characters")
    mutation_locality = raw.get("mutation_locality")
    if mutation_locality not in {None, "distributed", "local"}:
        raise CorpusError("mutation_locality must be local, distributed, or null")

    return Recipe(
        recipe_id=recipe_id,
        seed=raw_seed,
        target_entries=effective_target,
        topology=topology,
        fanout=_require_int("fanout", raw.get("fanout"), minimum=1),
        max_depth=_require_int("max_depth", raw.get("max_depth"), minimum=1),
        metadata_profile=metadata_profile,
        optional_links=optional_links,
        transitions=transitions,
        mutation_locality=mutation_locality,
        scale_points=scale_points,
    )


def _generate_contract(
    root: Path,
    recipe: Recipe,
    builder: _ManifestBuilder,
    capabilities: Dict[str, Any],
) -> None:
    directories = ("empty", "nested", "nested/deeper")
    files = (
        (".hidden", 5),
        ("UPPER.JSON", 33),
        ("archive.tar.gz", 64),
        ("empty.txt", 0),
        ("no-extension", 11),
        ("space name.txt", 17),
        ("tiny.py", 7),
        ("nested/data.json", 257),
        ("nested/readme.md", 1_024),
        ("nested/deeper/blob.bin", 4_096),
        ("nested/deeper/zero", 0),
    )
    if len(directories) + len(files) != recipe.target_entries:
        raise CorpusError("the committed contract recipe and target count disagree")
    builder.add(_directory_record(".", recipe.seed))
    for relative in directories:
        (root / relative).mkdir()
        builder.add(_directory_record(relative, recipe.seed))
    source: Optional[Tuple[str, int, int]] = None
    for relative, size in files:
        mtime = _create_file(root, relative, size, recipe.seed)
        builder.add(_file_record(relative, size, mtime))
        if relative == "nested/data.json":
            source = (relative, size, mtime)
    if source is None:
        raise CorpusError("contract hardlink source is missing")
    _create_optional_links(root, recipe, builder, capabilities, source)


def _generate_parametric(
    root: Path,
    recipe: Recipe,
    builder: _ManifestBuilder,
    capabilities: Dict[str, Any],
) -> None:
    builder.add(_directory_record(".", recipe.seed))
    directory_count = _directory_count(recipe)
    directories: List[str] = ["."]
    for index in range(directory_count):
        parent = _directory_parent(recipe, directories, index)
        name = (
            f"d{index:02x}"
            if recipe.topology == "deep"
            else f"d-{index:07d}-{_stable_token(recipe.seed, 'directory', index, 6)}"
        )
        relative = name if parent == "." else f"{parent}/{name}"
        (root / relative).mkdir()
        directories.append(relative)
        builder.add(_directory_record(relative, recipe.seed))

    source: Optional[Tuple[str, int, int]] = None
    file_count = recipe.target_entries - directory_count
    for index in range(file_count):
        parent = _file_parent(recipe, directories, index)
        extension = _file_extension(recipe, index)
        suffix = "" if extension == "<none>" else extension
        name = f"f-{index:09d}-{_stable_token(recipe.seed, 'file', index, 8)}{suffix}"
        relative = name if parent == "." else f"{parent}/{name}"
        size = _file_size(recipe, index)
        mtime = _create_file(root, relative, size, recipe.seed)
        builder.add(_file_record(relative, size, mtime))
        if source is None:
            source = (relative, size, mtime)
    if source is not None:
        _create_optional_links(root, recipe, builder, capabilities, source)


def _create_optional_links(
    root: Path,
    recipe: Recipe,
    builder: _ManifestBuilder,
    capabilities: Dict[str, Any],
    source: Tuple[str, int, int],
) -> None:
    source_path, source_size, source_mtime = source
    if "hardlink" in recipe.optional_links:
        relative = "zz-optional-hardlink.bin"
        try:
            os.link(root / source_path, root / relative)
        except (NotImplementedError, OSError) as error:
            capabilities["hardlink"] = _unsupported_capability(error)
        else:
            builder.add(
                _file_record(
                    relative,
                    source_size,
                    source_mtime,
                    hardlink_to=source_path,
                )
            )
            capabilities["hardlink"] = {
                "entries_added": 1,
                "reason": None,
                "requested": True,
                "supported": True,
            }
    if "symlink" in recipe.optional_links:
        relative = "zz-optional-symlink"
        try:
            os.symlink(source_path, root / relative)
        except (NotImplementedError, OSError) as error:
            capabilities["symlink"] = _unsupported_capability(error)
        else:
            builder.add({"kind": "symlink", "path": relative, "target": source_path})
            capabilities["symlink"] = {
                "entries_added": 1,
                "reason": None,
                "requested": True,
                "supported": True,
            }


def _observe_corpus(
    root: Path, *, capture_records: bool
) -> Tuple[Dict[str, Any], Dict[str, Any]]:
    if root.is_symlink() or not root.is_dir():
        raise CorpusError("corpus root must be a real directory")
    hardlinks: Dict[Tuple[int, int], Tuple[str, int]] = {}
    for _path, relative, metadata in _walk_corpus(root):
        if stat.S_ISREG(metadata.st_mode):
            identity = (metadata.st_dev, metadata.st_ino)
            previous = hardlinks.get(identity)
            canonical = min(previous[0], relative) if previous else relative
            hardlinks[identity] = (canonical, (previous[1] if previous else 0) + 1)

    builder = _ManifestBuilder(capture_records=capture_records)
    engine_digest = _SemanticAccumulator(algorithm=ENGINE_DIGEST_ALGORITHM)
    engine_digest.add_bytes(_engine_record_bytes(".", "directory", None))
    rollups = _RollupAccumulator()
    root_metadata = root.lstat()
    if not stat.S_ISDIR(root_metadata.st_mode):
        raise CorpusError("corpus root changed while it was being observed")
    builder.add(
        {
            "kind": "directory",
            "mtime_ns": root_metadata.st_mtime_ns,
            "path": ".",
        }
    )
    allocated_supported = hasattr(root_metadata, "st_blocks")
    entry_allocated_bytes = 0
    unique_allocated_bytes = 0
    allocated_identities: set[Tuple[int, int]] = set()
    for path, relative, metadata in _walk_corpus(root):
        mode = metadata.st_mode
        if stat.S_ISDIR(mode):
            engine_kind = "directory"
            record = {
                "kind": "directory",
                "mtime_ns": metadata.st_mtime_ns,
                "path": relative,
            }
        elif stat.S_ISREG(mode):
            engine_kind = "file"
            identity = (metadata.st_dev, metadata.st_ino)
            canonical, occurrences = hardlinks.get(identity, (relative, 1))
            hardlink_to = (
                canonical if occurrences > 1 and relative != canonical else None
            )
            record = _file_record(
                relative,
                metadata.st_size,
                metadata.st_mtime_ns,
                hardlink_to=hardlink_to,
            )
            if allocated_supported:
                allocated = int(getattr(metadata, "st_blocks")) * 512
                entry_allocated_bytes += allocated
                if identity not in allocated_identities:
                    unique_allocated_bytes += allocated
                    allocated_identities.add(identity)
        elif stat.S_ISLNK(mode):
            engine_kind = "symlink"
            record = {
                "kind": "symlink",
                "path": relative,
                "target": _normalize_link_target(os.readlink(path)),
            }
        else:
            engine_kind = "other"
            record = {"kind": "other", "path": relative}
        engine_digest.add_bytes(
            _engine_record_bytes(relative, engine_kind, metadata)
        )
        rollups.add(relative, engine_kind, metadata)
        builder.add(record)

    summary = builder.summary()
    summary["engine_digest"] = engine_digest.finish()
    summary["engine_digest_components"] = engine_digest.components()
    summary.update(rollups.summary())
    summary["sizes"].update(
        {
            "entry_allocated_bytes": entry_allocated_bytes
            if allocated_supported
            else None,
            "unique_allocated_bytes": unique_allocated_bytes
            if allocated_supported
            else None,
        }
    )
    capability = {
        "allocated_size": {
            "reason": (
                None if allocated_supported else "os.stat_result has no st_blocks"
            ),
            "supported": allocated_supported,
        }
    }
    return summary, capability


def _engine_record_bytes(
    relative: str,
    kind: str,
    metadata: Optional[os.stat_result],
) -> bytes:
    path_bytes = relative.encode("utf-8")
    if len(path_bytes) > (1 << 32) - 1:
        raise CorpusError("engine oracle path exceeds its u32 encoding")
    kind_value = {
        "file": 0,
        "directory": 1,
        "symlink": 2,
        "other": 3,
    }[kind]
    if metadata is None:
        attrs = (0, 0, 0, 0, 0, 0)
    else:
        size = int(metadata.st_size)
        allocated = (
            int(metadata.st_blocks) * 512
            if os.name == "posix" and hasattr(metadata, "st_blocks")
            else size
        )
        attrs = (
            size,
            allocated,
            int(metadata.st_mtime_ns),
            int(metadata.st_ctime_ns) if os.name == "posix" else 0,
            int(metadata.st_ino) if os.name == "posix" else 0,
            int(metadata.st_dev) if os.name == "posix" else 0,
        )
    return b"".join(
        (
            struct.pack(">I", len(path_bytes)),
            path_bytes,
            bytes((kind_value,)),
            struct.pack(">QQqqQQ", *attrs),
        )
    )


def _empty_rollup() -> Dict[str, Any]:
    return {
        "allocated": 0,
        "by_ext": {},
        "bytes": 0,
        "dirs": 0,
        "files": 0,
        "newest_mtime_ns": None,
    }


def _ancestor_directories(relative: str) -> Iterator[str]:
    current = str(PurePosixPath(relative).parent)
    while True:
        yield current
        if current == ".":
            return
        current = str(PurePosixPath(current).parent)


def _rollup_extension(relative: str) -> Optional[str]:
    name = PurePosixPath(relative).name
    searchable = name[1:] if name.startswith(".") else name
    dot = searchable.rfind(".")
    if dot < 0:
        return None
    stem = searchable[:dot]
    last = searchable[dot:]
    if len(last) <= 1:
        return None
    inner_dot = stem.rfind(".")
    if inner_dot >= 0 and stem[inner_dot:].lower() == ".tar":
        return f".tar{last.lower()}"
    return last.lower()


def _rollup_record_bytes(relative: str, rollup: Mapping[str, Any]) -> bytes:
    path_bytes = relative.encode("utf-8")
    if len(path_bytes) > (1 << 32) - 1:
        raise CorpusError("roll-up oracle path exceeds its u32 encoding")
    extensions = rollup["by_ext"]
    if len(extensions) > (1 << 32) - 1:
        raise CorpusError("roll-up oracle extension count exceeds u32")
    newest = rollup["newest_mtime_ns"]
    parts = [
        struct.pack(">I", len(path_bytes)),
        path_bytes,
        struct.pack(
            ">QQQQqI",
            rollup["files"],
            rollup["dirs"],
            rollup["bytes"],
            rollup["allocated"],
            0 if newest is None else newest,
            len(extensions),
        ),
    ]
    for extension, tally in sorted(extensions.items()):
        extension_bytes = extension.encode("utf-8")
        if len(extension_bytes) > (1 << 32) - 1:
            raise CorpusError("roll-up oracle extension exceeds u32")
        parts.extend(
            (
                struct.pack(">I", len(extension_bytes)),
                extension_bytes,
                struct.pack(">QQ", tally["files"], tally["bytes"]),
            )
        )
    return b"".join(parts)


def _walk_corpus(root: Path) -> Iterator[Tuple[Path, str, os.stat_result]]:
    pending: List[Tuple[Path, str]] = [(root, ".")]
    while pending:
        directory, relative_directory = pending.pop()
        try:
            with os.scandir(directory) as directory_entries:
                entries = sorted(directory_entries, key=lambda entry: entry.name)
        except OSError as error:
            raise CorpusError(
                f"cannot enumerate corpus directory: {error.strerror}"
            ) from error
        child_directories: List[Tuple[Path, str]] = []
        for entry in entries:
            relative = (
                entry.name
                if relative_directory == "."
                else f"{relative_directory}/{entry.name}"
            )
            path = Path(entry.path)
            try:
                metadata = _metadata_for_observation(entry, path)
            except OSError as error:
                raise CorpusError(
                    f"cannot stat corpus entry {relative!r}: {error.strerror}"
                ) from error
            yield path, relative, metadata
            if stat.S_ISDIR(metadata.st_mode):
                child_directories.append((path, relative))
        pending.extend(reversed(child_directories))


def _metadata_for_observation(entry: Any, path: Path) -> os.stat_result:
    """Return non-following metadata with authoritative file identity."""
    if _DIRENTRY_METADATA_IS_AUTHORITATIVE:
        return entry.stat(follow_symlinks=False)
    return path.lstat()


def _set_directory_mtimes(root: Path, seed: str) -> None:
    directories = [
        (path, relative)
        for path, relative, metadata in _walk_corpus(root)
        if stat.S_ISDIR(metadata.st_mode)
    ]
    directories.append((root, "."))
    for path, relative in sorted(
        directories, key=lambda item: _path_depth(item[1]), reverse=True
    ):
        mtime_ns = _mtime_ns_for(seed, relative)
        _set_path_times_ns(path, mtime_ns, mtime_ns)


def _set_path_times_ns(path: Path, atime_ns: int, mtime_ns: int) -> None:
    """Set exact times on an owned corpus path, rejecting observed symlinks."""
    timestamps = (atime_ns, mtime_ns)
    if os.utime in os.supports_follow_symlinks:
        os.utime(path, ns=timestamps, follow_symlinks=False)
        return

    # Python exposes the keyword on every platform but raises NotImplementedError when
    # `False` is unsupported. Generated files and directories are real paths; reject a
    # replaced symlink before using the platform's ordinary path operation.
    if stat.S_ISLNK(path.lstat().st_mode):
        raise CorpusError(f"cannot set deterministic times through symlink: {path}")
    os.utime(path, ns=timestamps)


def _create_file(root: Path, relative: str, size: int, seed: str) -> int:
    path = root / relative
    token = hashlib.sha256(f"{seed}\0payload\0{relative}".encode("utf-8")).digest()
    with path.open("xb") as output:
        if size <= 4_096:
            output.write((token * ((size + len(token) - 1) // len(token)))[:size])
        elif size > 0:
            output.write((token * 2)[: min(size, len(token) * 2)])
            output.truncate(size)
    mtime_ns = _mtime_ns_for(seed, relative)
    _set_path_times_ns(path, mtime_ns, mtime_ns)
    return mtime_ns


def _directory_record(relative: str, seed: str) -> Dict[str, Any]:
    return {
        "kind": "directory",
        "mtime_ns": _mtime_ns_for(seed, relative),
        "path": relative,
    }


def _file_record(
    relative: str,
    size: int,
    mtime_ns: int,
    *,
    hardlink_to: Optional[str] = None,
) -> Dict[str, Any]:
    record: Dict[str, Any] = {
        "kind": "file",
        "mtime_ns": mtime_ns,
        "path": relative,
        "size": size,
    }
    if hardlink_to is not None:
        record["hardlink_to"] = hardlink_to
    return record


def _directory_count(recipe: Recipe) -> int:
    if recipe.target_entries == 1:
        return 0
    if recipe.topology == "deep":
        return min(recipe.target_entries - 1, recipe.max_depth)
    if recipe.topology == "wide":
        return min(
            recipe.target_entries - 1,
            max(1, min(recipe.fanout, recipe.target_entries // 8)),
        )
    return min(recipe.target_entries - 1, max(1, recipe.target_entries // 8))


def _directory_parent(recipe: Recipe, directories: Sequence[str], index: int) -> str:
    if recipe.topology == "deep":
        return directories[-1]
    if recipe.topology == "wide":
        return "."
    return directories[index // recipe.fanout]


def _file_parent(recipe: Recipe, directories: Sequence[str], index: int) -> str:
    if len(directories) == 1:
        return "."
    if recipe.mutation_locality == "local":
        return directories[1]
    if recipe.topology == "wide":
        return directories[1 + index % (len(directories) - 1)]
    if recipe.topology == "deep":
        return directories[1 + index % (len(directories) - 1)]
    selected = _stable_index(recipe.seed, "file-parent", index, len(directories))
    return directories[selected]


def _file_extension(recipe: Recipe, index: int) -> str:
    if recipe.metadata_profile == "mixed":
        choices = (".bin", ".json", ".log", ".txt", "<none>")
    elif recipe.metadata_profile == "partial":
        choices = (".dat", ".txt", "<none>")
    else:
        choices = (".bin", ".json", ".md", ".txt", "<none>")
    return choices[_stable_index(recipe.seed, "extension", index, len(choices))]


def _file_size(recipe: Recipe, index: int) -> int:
    if recipe.metadata_profile == "empty":
        return 0
    if recipe.metadata_profile == "mixed":
        choices = (0, 1, 7, 64, 1_024, 4_096, 65_537, 1_048_576)
    else:
        choices = (0, 7, 64, 257, 1_024, 4_096)
    return choices[_stable_index(recipe.seed, "size", index, len(choices))]


def _initial_capabilities(recipe: Recipe) -> Dict[str, Any]:
    capabilities: Dict[str, Any] = {}
    for link_kind in sorted(_OPTIONAL_LINKS):
        requested = link_kind in recipe.optional_links
        capabilities[link_kind] = {
            "entries_added": 0,
            "reason": "not requested" if not requested else "not attempted",
            "requested": requested,
            "supported": False,
        }
    capabilities["partial_permissions"] = {
        "reason": (
            "portable baseline only; this host profile did not request permission "
            "manipulation"
            if recipe.metadata_profile == "partial"
            else "not requested"
        ),
        "requested": recipe.metadata_profile == "partial",
        "supported": False,
    }
    return capabilities


def _unsupported_capability(error: BaseException) -> Dict[str, Any]:
    errno_value = getattr(error, "errno", None)
    reason = type(error).__name__
    if errno_value is not None:
        reason = f"{reason} errno={errno_value}"
    return {
        "entries_added": 0,
        "reason": reason,
        "requested": True,
        "supported": False,
    }


def _assert_expected_matches_observed(
    expected: Mapping[str, Any], observed: Mapping[str, Any]
) -> None:
    for field in (
        "counts",
        "extensions",
        "semantic_components",
        "semantic_digest",
        "topology",
    ):
        if expected.get(field) != observed.get(field):
            raise CorpusError(
                f"creation ledger disagrees with observation for {field}: "
                f"expected {expected.get(field)!r}, observed {observed.get(field)!r}"
            )
    for field in ("entry_apparent_bytes", "unique_apparent_bytes"):
        if expected["sizes"].get(field) != observed["sizes"].get(field):
            raise CorpusError(
                f"creation ledger disagrees with observation for sizes.{field}"
            )
    if "records" in expected and expected.get("records") != observed.get("records"):
        raise CorpusError("creation ledger disagrees with observation for records")


@contextmanager
def _operation_lock(run_root: Path) -> Iterator[None]:
    lock_path = run_root / ".fdu-perf-operation-lock"
    try:
        lock_path.mkdir()
    except FileExistsError as error:
        raise CorpusError(
            "another corpus operation is already active or left a stale operation lock"
        ) from error
    except OSError as error:
        raise CorpusError(
            f"cannot acquire the corpus operation lock: {error}"
        ) from error
    try:
        yield
    finally:
        try:
            lock_path.rmdir()
        except FileNotFoundError:
            pass


def _validate_run_root(run_root: Path) -> Path:
    candidate = Path(run_root)
    if candidate.is_symlink():
        raise CorpusError("run directory may not be a symlink")
    try:
        resolved = candidate.resolve(strict=True)
    except FileNotFoundError as error:
        raise CorpusError("run directory does not exist") from error
    if not resolved.is_dir():
        raise CorpusError("run root is not a directory")
    if resolved == PROJECT_ROOT or resolved in PROJECT_ROOT.parents:
        raise CorpusError("refusing to operate on the workspace or one of its parents")
    if not resolved.name.startswith(RUN_PREFIX):
        raise CorpusError(f"run directory name must start with {RUN_PREFIX!r}")
    marker_path = resolved / MARKER_NAME
    if marker_path.is_symlink() or not marker_path.is_file():
        raise CorpusError("run directory marker is missing or is a symlink")
    marker = _load_json(marker_path)
    expected = {
        "resolved_path": str(resolved),
        "run_id": resolved.name,
        "schema": MARKER_SCHEMA,
    }
    if marker != expected:
        raise CorpusError("run directory marker does not identify this exact directory")
    return resolved


def _load_manifest(path: Path) -> Dict[str, Any]:
    manifest = _load_json(path)
    if not isinstance(manifest, dict):
        raise CorpusError("observed corpus manifest is not an object")
    if manifest.get("schema") != MANIFEST_SCHEMA:
        raise CorpusError(
            f"unsupported observed corpus schema: {manifest.get('schema')!r}"
        )
    recorded_hash = manifest.get("manifest_hash")
    if not isinstance(recorded_hash, str) or recorded_hash != _manifest_hash(manifest):
        raise CorpusError("observed corpus manifest hash is invalid")
    _validate_manifest_shape(manifest)
    return manifest


def _validate_manifest_shape(manifest: Mapping[str, Any]) -> None:
    required = {
        "capabilities",
        "counts",
        "extensions",
        "generator_source_hash",
        "generator_version",
        "manifest_hash",
        "oracle",
        "recipe",
        "recipe_hash",
        "recipe_id",
        "schema",
        "semantic_digest",
        "sizes",
        "state",
        "topology",
    }
    allowed = required | {"records", "transition_history"}
    unknown = set(manifest) - allowed
    missing = required - set(manifest)
    if unknown or missing:
        raise CorpusError(
            f"observed corpus manifest has unknown fields {sorted(unknown)} "
            f"or missing fields {sorted(missing)}"
        )
    for field in (
        "capabilities",
        "counts",
        "extensions",
        "oracle",
        "recipe",
        "sizes",
        "state",
        "topology",
    ):
        if not isinstance(manifest.get(field), dict):
            raise CorpusError(
                f"observed corpus manifest field {field!r} must be an object"
            )
    for field in (
        "generator_source_hash",
        "manifest_hash",
        "recipe_hash",
        "semantic_digest",
    ):
        _require_sha256(field, manifest.get(field))
    if not isinstance(manifest.get("recipe_id"), str) or not manifest["recipe_id"]:
        raise CorpusError(
            "observed corpus manifest recipe_id must be a non-empty string"
        )
    if manifest.get("generator_version") != GENERATOR_VERSION:
        raise CorpusError(
            "unsupported corpus generator version: "
            f"{manifest.get('generator_version')!r}"
        )
    state = manifest["state"]
    if not isinstance(state.get("id"), str) or not isinstance(
        state.get("transition_index"), int
    ):
        raise CorpusError("observed corpus manifest transition state is malformed")
    oracle = manifest["oracle"]
    if oracle.get("digest_algorithm") != SEMANTIC_DIGEST_ALGORITHM:
        raise CorpusError("observed corpus manifest digest algorithm is unsupported")
    if oracle.get("engine_digest_algorithm") != ENGINE_DIGEST_ALGORITHM:
        raise CorpusError("observed corpus manifest engine digest is unsupported")
    if oracle.get("rollup_digest_algorithm") != ROLLUP_DIGEST_ALGORITHM:
        raise CorpusError("observed corpus manifest roll-up digest is unsupported")
    components = oracle.get("semantic_components")
    accumulator = _SemanticAccumulator(
        components if isinstance(components, Mapping) else None
    )
    if not isinstance(components, Mapping):
        raise CorpusError("observed corpus manifest semantic components are malformed")
    if accumulator.finish() != manifest["semantic_digest"]:
        raise CorpusError(
            "observed corpus semantic digest does not match its components"
        )
    engine_components = oracle.get("engine_digest_components")
    engine_accumulator = _SemanticAccumulator(
        engine_components if isinstance(engine_components, Mapping) else None,
        algorithm=ENGINE_DIGEST_ALGORITHM,
    )
    if not isinstance(engine_components, Mapping):
        raise CorpusError("observed corpus engine digest components are malformed")
    if engine_accumulator.finish() != oracle.get("engine_digest"):
        raise CorpusError("observed corpus engine digest does not match its components")
    rollup_components = oracle.get("rollup_digest_components")
    rollup_accumulator = _SemanticAccumulator(
        rollup_components if isinstance(rollup_components, Mapping) else None,
        algorithm=ROLLUP_DIGEST_ALGORITHM,
    )
    if not isinstance(rollup_components, Mapping):
        raise CorpusError("observed corpus roll-up digest components are malformed")
    if rollup_accumulator.finish() != oracle.get("rollup_digest"):
        raise CorpusError("observed corpus roll-up digest does not match its components")
    records = manifest.get("records")
    if records is not None and not isinstance(records, list):
        raise CorpusError("observed corpus manifest records must be an array")
    history = manifest.get("transition_history")
    if history is not None and not isinstance(history, list):
        raise CorpusError(
            "observed corpus manifest transition history must be an array"
        )


def _manifest_hash(manifest: Mapping[str, Any]) -> str:
    unhashed = {key: value for key, value in manifest.items() if key != "manifest_hash"}
    return _sha256_json(unhashed)


def _load_json(path: Path) -> Any:
    try:
        if path.stat().st_size > MAX_JSON_BYTES:
            raise CorpusError(
                f"JSON input {path.name} exceeds the {MAX_JSON_BYTES}-byte safety limit"
            )
        with path.open("r", encoding="utf-8") as input_file:
            return json.load(input_file, parse_constant=_reject_json_constant)
    except (OSError, json.JSONDecodeError) as error:
        raise CorpusError(
            f"cannot read valid JSON from {path.name}: {error}"
        ) from error


def _reject_json_constant(value: str) -> None:
    raise json.JSONDecodeError(f"non-standard JSON constant {value!r}", value, 0)


def _atomic_write_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        dir=str(path.parent), prefix=f".{path.name}.", suffix=".tmp"
    )
    temporary_path = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as output:
            json.dump(value, output, ensure_ascii=True, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary_path, path)
    except Exception:
        try:
            temporary_path.unlink()
        except FileNotFoundError:
            pass
        raise


def _canonical_json(value: Mapping[str, Any]) -> bytes:
    return json.dumps(
        value, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")


def _sha256_json(value: Mapping[str, Any]) -> str:
    return hashlib.sha256(_canonical_json(value)).hexdigest()


def _stable_token(seed: str, namespace: str, index: int, length: int) -> str:
    digest = hashlib.sha256(f"{seed}\0{namespace}\0{index}".encode("utf-8")).hexdigest()
    return digest[:length]


def _stable_index(seed: str, namespace: str, index: int, length: int) -> int:
    if length <= 0:
        raise CorpusError("stable choice requires at least one option")
    digest = hashlib.sha256(f"{seed}\0{namespace}\0{index}".encode("utf-8")).digest()
    return int.from_bytes(digest[:8], "big") % length


def _mutation_candidates(
    root: Path, recipe: Recipe, transition_id: str
) -> List[Tuple[str, int, int]]:
    candidates: List[Tuple[str, int, int]] = []
    parent_counts: Dict[str, int] = {}
    for _path, relative, metadata in _walk_corpus(root):
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            continue
        candidates.append((relative, metadata.st_size, metadata.st_mtime_ns))
        parent = str(PurePosixPath(relative).parent)
        parent_counts[parent] = parent_counts.get(parent, 0) + 1
    if recipe.mutation_locality == "local" and parent_counts:
        local_parent = min(
            parent_counts,
            key=lambda parent: (-parent_counts[parent], parent),
        )
        candidates = [
            candidate
            for candidate in candidates
            if str(PurePosixPath(candidate[0]).parent) == local_parent
        ]
    candidates.sort(
        key=lambda candidate: hashlib.sha256(
            f"{recipe.seed}\0mutation:{transition_id}\0{candidate[0]}".encode("utf-8")
        ).digest()
    )
    return candidates


def _transition_change_count(transition_id: str, available_files: int) -> int:
    if transition_id == "one-change":
        return 1
    one_percent = max(1, math.ceil(available_files / 100))
    if transition_id == "mixed-1pct":
        return max(3, one_percent)
    return one_percent


def _mutation_destination(
    relative: str,
    *,
    transition_index: int,
    operation_index: int,
    seed: str,
    prefix: str,
) -> str:
    path = PurePosixPath(relative)
    suffix = path.suffix
    token = _stable_token(
        seed,
        f"mutation-destination:{transition_index}:{prefix}",
        operation_index,
        10,
    )
    name = f"m-{transition_index}-{prefix}-{operation_index:06d}-{token}{suffix}"
    parent = str(path.parent)
    return name if parent == "." else f"{parent}/{name}"


def _mutation_mtime_ns(
    seed: str, transition_index: int, relative: str, operation_index: int
) -> int:
    offset = _stable_index(
        seed,
        f"mutation-mtime:{transition_index}:{relative}",
        operation_index,
        10_000,
    )
    seconds = BASE_MTIME_SECONDS + 100_000 + transition_index * 20_000 + offset * 2
    return seconds * 1_000_000_000


def _rewrite_file_preserving_size(path: Path, mtime_ns: int, seed: str) -> None:
    size = path.stat().st_size
    if size > 0:
        replacement = hashlib.sha256(
            f"{seed}\0mutation-payload\0{path.name}\0{mtime_ns}".encode("utf-8")
        ).digest()[:1]
        with path.open("r+b") as output:
            output.write(replacement)
    _set_path_times_ns(path, mtime_ns, mtime_ns)


def _mtime_ns_for(seed: str, relative: str) -> int:
    offset = _stable_index(seed, f"mtime:{relative}", 0, 10_000)
    return (BASE_MTIME_SECONDS + offset * 2) * 1_000_000_000


def _generator_source_hash() -> str:
    digest = hashlib.sha256()
    for source in (Path(__file__), Path(__file__).with_name("corpora.json")):
        digest.update(source.name.encode("utf-8"))
        digest.update(b"\0")
        with source.open("rb") as input_file:
            for chunk in iter(lambda: input_file.read(64 * 1024), b""):
                digest.update(chunk)
    return digest.hexdigest()


def _extension_for(relative: str) -> str:
    suffix = PurePosixPath(relative).suffix.lower()
    return suffix if suffix else "<none>"


def _normalize_link_target(target: str) -> str:
    return target.replace("\\", "/") if os.sep == "\\" else target


def _path_depth(relative: str) -> int:
    return 0 if relative == "." else len(PurePosixPath(relative).parts)


def _require_relative_record_path(value: Any) -> str:
    if not isinstance(value, str) or not value:
        raise CorpusError("record path must be a non-empty string")
    path = PurePosixPath(value)
    if value != "." and (
        path.is_absolute() or ".." in path.parts or str(path) != value
    ):
        raise CorpusError(f"record path is not normalized and relative: {value!r}")
    return value


def _require_int(name: str, value: Any, minimum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
        raise CorpusError(
            f"{name} must be an integer greater than or equal to {minimum}"
        )
    return value


def _require_sha256(name: str, value: Any) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise CorpusError(f"{name} must be a lowercase SHA-256 digest")
    try:
        parsed = bytes.fromhex(value)
    except ValueError as error:
        raise CorpusError(f"{name} must be a lowercase SHA-256 digest") from error
    if len(parsed) != 32 or value != value.lower():
        raise CorpusError(f"{name} must be a lowercase SHA-256 digest")
    return value


def _require_choice(name: str, value: Any, choices: set[str]) -> str:
    if not isinstance(value, str) or value not in choices:
        raise CorpusError(f"{name} must be one of: {', '.join(sorted(choices))}")
    return value


def _require_string_sequence(
    name: str, value: Any, choices: set[str]
) -> Tuple[str, ...]:
    if not isinstance(value, list) or any(
        not isinstance(item, str) or item not in choices for item in value
    ):
        raise CorpusError(f"{name} contains an unsupported value")
    if len(value) != len(set(value)):
        raise CorpusError(f"{name} contains duplicate values")
    return tuple(value)
