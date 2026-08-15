---
type: is
id: is-01m01ecbhsetn1rmvfn8m26w7e
title: "H88: Separate macOS bulk, fallback, and topology scheduling signals"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:34.680Z
updated_at: 2026-08-15T11:06:42.894Z
closed_at: 2026-08-15T11:06:42.894Z
close_reason: Measured backend/topology signals across generated shapes. The frozen subject used macOS bulk for 60,314 directories with zero fallback; real completion order diverged from nominal generated phases, establishing that trace-defined service/frontier observations are required.
---
H88 is a signal-quality study, not a second controller-selection bead. Determine whether one observation set can represent both the macOS bulk path and portable fallback by comparing directories, entries-per-directory, bulk calls, fallback counts, completed versus in-flight service, ready-frontier width, handoff backlog/high-water, and achieved useful throughput across the deterministic wide, deep, many-small-directory, mixed-phase, mount/firmlink-boundary, and permission diagnostics.

Acceptance: each candidate signal is assessed for observability, overhead, causal usefulness, backend portability, and failure modes; no result changes semantic fallback behavior or introduces topology-specific constants; the output recommends the smallest signal set, or records that backend-aware observations are necessary, for fdu-9x4o to prototype; rejected signals and every measured regime are recorded in the ledger. Controller design and production code remain outside this bead.
