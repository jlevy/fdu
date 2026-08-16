---
type: is
id: is-01m03y2wrr57zxh6g5n7zsj8n3
title: Add a perf-harness job for the default one-shot CLI plan
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md
labels:
  - performance
dependencies: []
parent_id: is-01m03bjey08898z8t9a2vhakm1
created_at: 2026-08-16T00:03:30.711Z
updated_at: 2026-08-16T00:03:30.711Z
---
The performance harness measures the library open paths (cold-scan-index,
warm-revalidate via perf_probe) and the transient-summary installed-command contract
(fdu-transient-summary). Neither covers the default one-shot CLI configuration --
FullIndex plan, auto policy -- which is what users actually run and where fdu-wpku
found and fixed the field-report regression. The read-gate change is invisible to
every existing harness job: perf_probe drives open(), which deliberately keeps the
warm path.

Needed: an installed-command contract for the default tree plan (fdu PATH), so changes
to the one-shot execution path can be measured and recorded through the ledger rather
than ad-hoc interleaved scripts. Related: fdu-5yjk (FDU_SCAN_DIAGNOSTICS cannot
instrument the FullIndex plan either -- same blind spot, observability side).
