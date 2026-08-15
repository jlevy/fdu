"""Strict downstream-consumer fixture; this file is type-checked, not executed."""

from pathlib import Path

import fdu

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
