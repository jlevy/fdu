"""Typed facade over the private native extension."""

from __future__ import annotations

import json
from collections.abc import Iterator, Sequence
from dataclasses import replace
from datetime import datetime
from pathlib import Path
from types import TracebackType
from typing import Any, cast

from . import _native
from ._models import (
    AnalysisOptions,
    Bound,
    CachePolicy,
    CacheStatus,
    Change,
    ChangeKind,
    ChangeSet,
    Child,
    EntryKind,
    Format,
    Provenance,
    Query,
    RefreshResult,
    Report,
    RollUp,
    ScanOptions,
    Status,
    WatchOptions,
    provenance_from_dict,
    report_from_dict,
    rollup_from_dict,
    status_from_dict,
)


class FduError(RuntimeError):
    """Base class for public fdu API errors."""


class InvalidArgumentError(FduError, ValueError):
    """A public option or query is invalid."""


class FilesystemError(OSError, FduError):
    """A filesystem failure that prevented an operation from producing a result.

    Reads the way the CLI reports the same failure, because it is the same failure::

        fdu: I/O error at missing: No such file or directory (os error 2)

    OSError's own rendering leads with ``[Errno None]`` for anything raised from the
    engine, since a Rust ``io::Error`` kind does not always carry an errno. That is worse
    than uninformative -- it states an error number that does not exist -- so the path and
    the reason are put first and the errno is shown only when there is one.
    """

    @property
    def path(self) -> Path | None:
        """The path whose operation failed, when the failure named one."""

        return Path(self.filename) if self.filename else None

    def __str__(self) -> str:
        # strerror already carries the operating system's own detail, so an errno
        # suffix would repeat it -- "No such file or directory (os error 2) (errno 2)".
        reason = self.strerror or "operation failed"
        if self.filename:
            return f"I/O error at {self.filename}: {reason}"
        return reason


def _bound(value: int | Bound | str | None) -> str | None:
    if value is None:
        return None
    return value.value if isinstance(value, Bound) else str(value)


def _when(value: datetime | str | None) -> str | None:
    if isinstance(value, datetime):
        # The engine's time grammar requires an explicit UTC offset because it has no
        # time-zone database of its own. Python does, and a naive datetime means local
        # time (as in `datetime.timestamp()`), so resolve it here instead of handing
        # the native layer a timestamp it must reject.
        aware = value.astimezone() if value.tzinfo is None else value
        return aware.isoformat()
    return value


def _call(function: Any, /, *args: object, **kwargs: object) -> Any:
    try:
        return function(*args, **kwargs)
    except OSError as error:
        raise FilesystemError(error.errno, error.strerror, error.filename) from error
    except ValueError as error:
        raise InvalidArgumentError(str(error)) from error
    except RuntimeError as error:
        raise FduError(str(error)) from error


def _query_kwargs(query: Query) -> dict[str, object]:
    selection = query.selection
    return {
        # `None` rather than an empty list: the binding distinguishes "the caller named
        # no view" -- derive one from the analyzers -- from "the caller named none", which
        # would be a report with no sections.
        "views": [view.value for view in query.views] or None,
        "include": list(selection.include),
        "exclude": list(selection.exclude),
        "min_size": str(selection.min_size) if selection.min_size is not None else None,
        "modified_since": _when(selection.modified_since),
        "modified_before": _when(selection.modified_before),
        "kind": [kind.value for kind in selection.kinds],
        "depth": _bound(selection.depth),
        "limit": _bound(selection.limit),
        "sort": selection.sort.value if selection.sort is not None else None,
        "reverse": selection.reverse,
        "size": selection.size.value,
        "words_per_page": query.words_per_page,
    }


def _cache_status(value: dict[str, Any]) -> CacheStatus:
    return CacheStatus(
        path=Path(value["path"]),
        bytes=int(value["bytes"]),
        content_bytes=(int(value["content_bytes"]) if value["content_bytes"] is not None else None),
        recognized=bool(value["recognized"]),
        root=Path(value["root"]) if value["root"] is not None else None,
        entries=int(value["entries"]) if value["entries"] is not None else None,
        max_depth=int(value["max_depth"]) if value["max_depth"] is not None else None,
        one_filesystem=(
            bool(value["one_filesystem"]) if value["one_filesystem"] is not None else None
        ),
    )


class Watch(Iterator[tuple[Change, ...]]):
    """Closable iterator of event-driven change batches."""

    __slots__ = ("_native",)

    def __init__(self, native: _native.Watch) -> None:
        self._native = native

    def __iter__(self) -> Watch:
        return self

    def __next__(self) -> tuple[Change, ...]:
        return tuple(_change(item) for item in next(self._native))

    def close(self) -> None:
        self._native.close()

    def __enter__(self) -> Watch:
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc_value: BaseException | None,
        _traceback: TracebackType | None,
    ) -> bool:
        self.close()
        return False


class Index:
    """A retained in-process index serving many structured queries."""

    __slots__ = ("_native",)

    def __init__(self, native: _native.Index) -> None:
        self._native = native

    @property
    def root(self) -> Path:
        return Path(self._native.root)

    @property
    def clock(self) -> int:
        return self._native.clock

    @property
    def status(self) -> Status:
        return status_from_dict(_call(self._native.status))

    def __len__(self) -> int:
        return len(self._native)

    def report(self, query: Query | None = None) -> Report:
        selected = query if query is not None else Query()
        kwargs = _query_kwargs(selected)
        raw = _call(self._native.report_json, **kwargs)
        wire: dict[str, Any] = json.loads(raw)
        report = report_from_dict(wire)
        errors = self.status.errors

        # `render` re-projects the retained index rather than rescanning, so binding the
        # same query here costs a projection and keeps `Report.render` on the report where
        # `as_dict` already lives.
        def renderer(format: str, color: bool) -> str:
            return _call(self._native.report_json, **kwargs, format=format, color=color)

        return replace(
            report,
            status=replace(report.status, errors=errors),
            _renderer=renderer,
        )

    def total(self) -> RollUp:
        return rollup_from_dict(_call(self._native.total), self.provenance())

    def rollup(self, path: str | Path) -> RollUp | None:
        raw = _call(self._native.rollup, path)
        return None if raw is None else rollup_from_dict(raw, self.provenance(path))

    def children(self, path: str | Path = Path()) -> tuple[Child, ...] | None:
        raw = _call(self._native.children, path)
        if raw is None:
            return None
        return tuple(
            Child(
                name=str(item["name"]),
                kind=EntryKind(item["kind"]),
                rollup=(
                    rollup_from_dict(item["rollup"], provenance_from_dict(item["provenance"]))
                    if item.get("rollup") is not None
                    else None
                ),
                bytes=int(item["bytes"]) if item.get("bytes") is not None else None,
                allocated=(int(item["allocated"]) if item.get("allocated") is not None else None),
                mtime_ns=int(item["mtime_ns"]) if item.get("mtime_ns") is not None else None,
                provenance=provenance_from_dict(item["provenance"]),
            )
            for item in raw
        )

    def provenance(self, path: str | Path = Path()) -> Provenance | None:
        value = _call(self._native.provenance, path)
        return None if value is None else provenance_from_dict(value)

    def refresh(self) -> RefreshResult:
        value = _call(self._native.refresh)
        return RefreshResult(
            inserted=int(value["inserted"]),
            updated=int(value["updated"]),
            removed=int(value["removed"]),
            unchanged=int(value["unchanged"]),
            stale=int(value["stale"]),
            clock=int(value["clock"]),
            status=status_from_dict(value),
        )

    def since(self, clock: int) -> ChangeSet:
        value = _call(self._native.since, clock)
        return ChangeSet(
            truncated=bool(value["truncated"]),
            clock=int(value["clock"]),
            changes=tuple(_change(item) for item in value["ops"]),
        )

    def watch(self, options: WatchOptions | None = None) -> Watch:
        selected = options if options is not None else WatchOptions()
        arguments = _query_kwargs(selected.query)
        arguments["interval"] = selected.interval
        native = _call(
            self._native.watch,
            **arguments,
        )
        return Watch(native)


def _change(value: dict[str, Any]) -> Change:
    raw_kind = str(value["op"])
    return Change(
        clock=int(value["clock"]),
        path=Path(value["path"]),
        kind=ChangeKind(raw_kind),
        entry_kind=EntryKind(value["kind"]) if value.get("kind") is not None else None,
        bytes=int(value["bytes"]) if value.get("bytes") is not None else None,
        allocated=int(value["allocated"]) if value.get("allocated") is not None else None,
        mtime_ns=int(value["mtime_ns"]) if value.get("mtime_ns") is not None else None,
        reason=str(value["reason"]) if value.get("reason") is not None else None,
    )


def open(
    root: str | Path,
    *,
    cache: CachePolicy = CachePolicy.AUTO,
    scan: ScanOptions | None = None,
    analysis: AnalysisOptions | None = None,
) -> Index:
    """Open a root using the requested cache policy, then return a retained index."""

    scan_options = scan if scan is not None else ScanOptions()
    analysis_options = analysis if analysis is not None else AnalysisOptions()
    native = _call(
        _native.open,
        root,
        cache=cache.value,
        max_depth=scan_options.max_depth,
        one_filesystem=scan_options.one_filesystem,
        analyze=str(analysis_options.analyze),
        analysis_workers=analysis_options.workers,
    )
    return Index(native)


def scan(
    root: str | Path,
    *,
    scan: ScanOptions | None = None,
    analysis: AnalysisOptions | None = None,
) -> Index:
    """Walk a root without reading or writing a snapshot cache."""

    scan_options = scan if scan is not None else ScanOptions()
    analysis_options = analysis if analysis is not None else AnalysisOptions()
    native = _call(
        _native.scan,
        root,
        max_depth=scan_options.max_depth,
        one_filesystem=scan_options.one_filesystem,
        analyze=str(analysis_options.analyze),
        analysis_workers=analysis_options.workers,
    )
    return Index(native)


def render_cache_status(statuses: Sequence[CacheStatus], format: Format = Format.TEXT) -> str:
    """Render cache statuses exactly as ``fdu --cache-status`` prints them.

    The same renderer the CLI uses, in every format, so a caller holding
    :class:`CacheStatus` values can print what fdu prints instead of inventing a layout
    that will drift from it.
    """

    return cast(
        str,
        _call(_native.render_cache_status, [str(status.path) for status in statuses], str(format)),
    )


def cache_path(root: str | Path) -> Path | None:
    value = _call(_native.cache_path, root)
    return None if value is None else Path(value)


def cache_status(root: str | Path) -> CacheStatus | None:
    value = _call(_native.cache_status, root)
    return None if value is None else _cache_status(value)


def list_caches(root: str | Path = Path()) -> tuple[CacheStatus, ...]:
    return tuple(_cache_status(value) for value in _call(_native.list_caches, root))


def clear_cache(root: str | Path) -> bool:
    return bool(_call(_native.clear_cache, root))


def clear_all_caches(root: str | Path = Path()) -> int:
    return int(_call(_native.clear_all_caches, root))


def _main() -> int:
    """Console-script boundary; argument parsing remains in the native CLI."""

    return _native.main()
