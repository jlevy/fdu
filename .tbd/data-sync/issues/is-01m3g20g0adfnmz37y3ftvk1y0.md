---
type: is
id: is-01m3g20g0adfnmz37y3ftvk1y0
title: Revise cross-process FSEvents replay design and validate with a committed probe
kind: task
status: closed
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-26T23:50:04.998Z
updated_at: 2026-09-27T00:45:38.714Z
started_at: 2026-09-26T23:50:39.200Z
closed_at: 2026-09-27T00:45:38.704Z
close_reason: "Completed in PR #131 (d607d489): reviewed reproducible cross-process probe, explicit opt-in journal freshness contract, reuse of watch build feature, and additional multi-root hour/day disk-history proposal. Full local make check, cross-lint, probe tests, and all CI checks passed. Production replay, long-gap acceptance, and end-to-end large-tree latency remain separately tracked."
resolution: null
duplicate_of: null
---
Revise research and active design to specify no resident fdu requirement, evidence limits, explicit journal-scoped freshness, cursor publication, persistence costs, and platform policy. Implement and review the reproducible exploration probe with one delegated agent; run feasible immediate restart/overlap tests and leave longer retention acceptance explicitly pending.

## Notes

2026-09-26: Drafts updated and probe reviewed; disk-heavy work relocated to external task scratch under local AGENTS instructions. Scope expanded by user to an additive multi-root disk-pressure workflow: Home/Applications/canonical temp roots, actual hour/day checkpoints, seconds-scale attribution of agent-build growth, bounded writes and visible coverage. Full handoff gate in progress; no shipped engine behavior changed.
