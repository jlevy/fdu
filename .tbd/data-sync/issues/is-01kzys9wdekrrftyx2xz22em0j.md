---
type: is
id: is-01kzys9wdekrrftyx2xz22em0j
title: Probe job for the transient summary tier, with a tallies oracle
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01kzy2qv7fkcwjcn3g8gas7g4m
created_at: 2026-08-14T00:03:44.686Z
updated_at: 2026-08-23T08:22:13.183Z
closed_at: 2026-08-23T08:22:13.182Z
close_reason: "Landed: perf_probe gains a 'summary' mode driving prepare_report on the transient plan, and measure.py gains the aggregate-summary job with Job.oracle='tallies'. The blocker recorded in this bead (prepare_report was pub(crate)) had already been removed by fdu-z7sp. component_ns now measurable: ~5ms below wall on a 5,838-entry subject, which is most of what exp-043/044 were arguing over. Verified end to end through a real paired run: 0 invalid samples, tallies oracle agrees, statistics are ledger-shaped."
---
The aggregate tier (RetainedState::Summary) has no perf_probe mode and no measure.py job, so it cannot be A/B measured under the accept rule and has no component_ns. It is reachable only through compare_tools.py driving the real CLI, so every layer-1 number carries process spawn, arg parsing, canonicalize and JSON rendering. exp-043 and exp-044 both resolved on wall changes of +0.67% and -1.15% while user CPU fell 40% and 50%, with no component timer available to tell dilution from truth. The blocker is structural: summarize_index builds the verification digest by walking the index, and this tier retains no index. Needs a tallies-based oracle (files, dirs, apparent bytes, allocated bytes, newest mtime) checked against the tree fingerprint, as compare_tools already does.

## Notes

2026-08-15 (Linux session): attempted this and hit a design decision that should be made deliberately, not by a benchmarking convenience.

The aggregate/transient-summary tier is only reachable through execution::prepare_report, which is pub(crate) inside a private 'mod execution'. perf_probe is an example, i.e. a separate crate that sees only public API, so it cannot construct that tier at all. exp-040..046's rich-summary jobs worked because prototype engine code was in-tree at the time; all of it was reverted.

Three ways forward, in preference order:
1. Measure the CLI binary rather than the probe. The accept rule's default metric is wall time, and the harness already collects wall + rusage around spawn/wait, so a job whose argv is 'fdu --cache off --view summary ROOT' yields a verdict-grade number today. What it loses is component_ns attribution and the probe's JSON summary, so the oracle would need to check the CLI's own --format json payload (the tool-comparison harness already hashes that stable payload, so the machinery exists). No API change.
2. A #[doc(hidden)] or feature-gated measurement entry point. Cheapest to implement, but it widens the surface of a planner the design principles deliberately keep opaque ('exposes no fast-mode flag'), so it needs an explicit decision.
3. Make the planner public as designed API. Largest change; only worth it if a real consumer wants it.

Recommend (1): it keeps the API closed and still unblocks fdu-tk1b/H76, whose whole question is a wall-time comparison against diskus on the scalar tier.

Related: the cold-open-save job added in bd9779d is the pattern to copy for probe-reachable tiers, and exp-059 used it.
