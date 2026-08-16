---
type: is
id: is-01m03y2wrr57zxh6g5n7zsj8n3
title: Add a perf-harness job for the default one-shot CLI plan
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md
labels:
  - performance
dependencies: []
parent_id: is-01m03bjey08898z8t9a2vhakm1
created_at: 2026-08-16T00:03:30.711Z
updated_at: 2026-08-16T17:44:28.074Z
closed_at: 2026-08-16T17:44:28.074Z
close_reason: Default-CLI measurement path landed as the fdu-default-tree installed-command contract, with cache isolation for cache-writing contracts. Verified end-to-end on a real subject with zero invalid samples and no disturbance of the operator's cache.
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

## Notes

Landed as an installed-command contract `fdu-default-tree` in compare_tools.CONTRACTS
(branch claude/perf-record-phase-a), not as a perf_probe job.

Why the contract and not a job: the gap is about the installed command. perf_probe
builds --no-default-features and drives the library open(), which deliberately keeps the
warm-revalidate path that the one-shot CLI no longer takes after fdu-wpku -- so a
perf_probe job would have measured a different execution plan than the one users get,
which is the exact failure this bead exists to close.

Contract: argv ("{binary}", "--color", "never", "{root}") -- no --cache, no --view, no
--depth, because the point is that a user typed none of them. work_class "default-tree".
Legal comparison anchor; deliberately not in FDU_SUMMARY_CONTRACTS, so it cannot carry a
held-out release claim (different work class from the summary cells).

New harness capability it required: ToolContract.writes_cache. Every other contract
passes --cache off, so the harness had never needed to care where a tool keeps state.
This one writes a snapshot per run, so cache-writing contracts now get an isolated
XDG_CACHE_HOME for the comparison and fail closed when none was provisioned -- otherwise
a run would measure against whatever the operator had cached and leave a snapshot of the
subject tree behind. Verified end-to-end: real cache directory unchanged across a full
run (42 files before and after), no subject snapshot in it, no temp directory leaked,
0 invalid samples.

Per-run rather than per-trial isolation is correct because the default plan does not read
what it wrote: every trial cold-scans and rewrites, so trials stay identical to each
other. That property is what makes this a stable job rather than a first-run-only
measurement.

Remaining in Phase B (promoting session findings to artifacts) is blocked on host
conditions, not on the harness -- see fdu-ow8y.
