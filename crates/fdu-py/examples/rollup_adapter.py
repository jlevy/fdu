"""Adapt fdu roll-ups into a downstream application's own wire model."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path

import fdu


@dataclass(frozen=True, slots=True)
class DirectorySummary:
    """Example application-owned shape; this is intentionally not an fdu model."""

    path: str
    files: int
    directories: int
    apparent_bytes: int
    allocated_bytes: int
    complete: bool
    freshness: str


def summarize(index: fdu.Index, path: Path = Path()) -> DirectorySummary | None:
    """Map one precomputed fdu directory roll-up without parsing CLI output."""
    rollup = index.rollup(path) if path != Path() else index.total()
    if rollup is None:
        return None
    return DirectorySummary(
        path=str(path),
        files=rollup.files,
        directories=rollup.dirs,
        apparent_bytes=rollup.bytes,
        allocated_bytes=rollup.allocated,
        complete=index.status.complete,
        freshness=index.status.freshness.value,
    )


def main() -> None:
    """Scan once, then serve multiple application-owned summaries."""
    index = fdu.scan(Path())
    rows = [
        summary
        for path in (Path(), Path("src"), Path("docs"))
        if (summary := summarize(index, path))
    ]
    print(json.dumps([asdict(row) for row in rows], indent=2))


if __name__ == "__main__":
    main()
