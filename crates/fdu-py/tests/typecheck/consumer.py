"""Strict downstream-consumer fixture; this file is type-checked, not executed."""

from pathlib import Path

import fdu
from fdu import opened

index: fdu.Index = fdu.open(
    Path("."),
    cache=fdu.CachePolicy.OFF,
    scan=fdu.ScanOptions(one_filesystem=True),
)
report: fdu.Report = index.report(
    fdu.Query(
        views=(fdu.View.SUMMARY, fdu.View.TYPES),
        selection=fdu.Selection(limit=10, size=fdu.SizeMetric.APPARENT),
    )
)

complete: bool = report.status.complete
freshness: fdu.Freshness = report.status.freshness
for section in report.sections:
    view: fdu.View = section.view
    print(view, complete, freshness)

rollup: fdu.RollUp = index.total()
file_count: int = rollup.files
print(file_count)

registry: str = '[[kind]]\nid = "notes"\nfamily = "prose"\nextensions = ["md"]\n'
live: opened.OpenedIndex = opened.OpenedIndex.open(
    Path("."), opened.OpenedOptions(max_files=10_000, type_rules=registry)
)
live_read: opened.ReadResponse = live.read(
    opened.Lookup("pyproject.toml"),
    opened.Tree(page=opened.Page(limit=20, max_work=10_000)),
    opened.Tree("src", depth=opened.Bound.ALL, include_ignored=False),
    opened.Tree(depth=2),
    opened.Flat(
        selection=opened.EntrySelection(
            query=fdu.Selection(kinds=(fdu.EntryKind.FILE,)),
            max_size=1_000_000,
            exclude_ignored=True,
            logical_extensions=(".py",),
        )
    ),
    opened.Diagnostics(),
)
live_cursor: opened.EngineVersion = live_read.change_cursor
poll: opened.ChangePoll = live.changes(live_cursor, timeout=0.1)
print(poll.state.phase)
live.close()
