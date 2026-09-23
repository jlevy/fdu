---
type: is
id: is-01m2pyea2jz71fks8v1zep7c21
title: "P2.2.3: JSON Lines through the answer walk; delete collapse"
kind: task
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyeaeaj7mqe04gk45j9xjd
  - type: blocks
    target: is-01m2pmrmvr6x2kz662jksackef
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:42.641Z
updated_at: 2026-09-23T08:14:06.608Z
started_at: 2026-09-20T04:35:01.934Z
closed_at: 2026-09-23T08:14:06.608Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `report_format.rs`: `emit_report` with `Field { name, presence: Presence }` and `Presence::{Always, Nullable, WhenLossy, WhenAnalyzer(AnalysisSet), WhenSet}`, the only declaration of document structure; the tree walk uses an explicit stack.
- JSON Lines through `JsonSink::line`; delete `collapse` (`:1271-1274`), which rewrites bracketed strings.

**Tests**

- Rewrite the JSON Lines test (`:2429`); add a JSON Lines test with `{ ` in a name.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
