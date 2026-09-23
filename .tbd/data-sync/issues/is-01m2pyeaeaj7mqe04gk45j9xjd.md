---
type: is
id: is-01m2pyeaeaj7mqe04gk45j9xjd
title: "P2.2.4: Pretty JSON through the walk with a declared layout"
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyeas8avwnpbxs4tnr9rq0
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:43.017Z
updated_at: 2026-09-23T08:14:06.616Z
started_at: 2026-09-20T04:35:01.943Z
closed_at: 2026-09-23T08:14:06.616Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `report_format.rs`: delete the JSON writers (`:466-471`, `:538-976`) and `json_count` (`:1717-1719`); pretty JSON through `JsonSink::pretty` with a declared layout.

**Tests**

- Golden diffs are whitespace only, proven by comparing parsed values before and after.
- Extend stack-safety and non-Unicode path tests (`:2957`, `:3175`, `:3192`, `:2465`, `:2543`).

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: byte-identical pretty JSON would need layout hints for today's accidents, so the plan accepts whitespace-only golden diffs verified by parsed equality.
