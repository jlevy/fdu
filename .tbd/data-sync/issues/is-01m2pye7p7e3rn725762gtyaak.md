---
type: is
id: is-01m2pye7p7e3rn725762gtyaak
title: "P2.1.1: The METRICS table: one MetricDef per metric, owned by a requestable unit"
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
    target: is-01m2pye80k4gebgn994ewchs53
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2phzn814exmf4ty5vw6zha0
hold: null
hold_until: null
created_at: 2026-09-17T05:46:40.198Z
updated_at: 2026-09-23T08:14:06.558Z
started_at: 2026-09-20T04:39:46.846Z
closed_at: 2026-09-23T08:14:06.558Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `content/content_model.rs`: replace `MetricSlotId` (`:18`, unused internally but re-exported at `content.rs:24`, so its removal is public) with `MetricDef { name, owner: AnalysisSet, analyzer: AnalyzerId, doc }` and `METRICS`: `lines` owns `physical_lines`, `blank_lines`, `nonblank_lines`, `raw_words`; `code` owns `code_lines`, `comment_lines`, `code_blank_lines`; `words` owns `logical_words`, `paragraphs`, `visible_words`, `visible_logical_words`, `document_words`.

**Tests**

- Every emitted metric key appears in `METRICS` exactly once.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
