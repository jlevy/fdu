---
type: is
id: is-01m2pye8ptnr6019rcb15g4w69
title: "P2.1.4: Metric presence follows the request under fdu.report/7"
kind: task
status: closed
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye91dbqqaf1qy70cem3mt
  - type: blocks
    target: is-01m2pyebe93530evdeaw8tcxh6
  - type: blocks
    target: is-01kzyqkgbvy3cmmc3qx9zwfnzp
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2phzn814exmf4ty5vw6zha0
hold: null
hold_until: null
created_at: 2026-09-17T05:46:41.242Z
updated_at: 2026-09-23T08:14:06.579Z
started_at: 2026-09-20T04:39:46.882Z
closed_at: 2026-09-23T08:14:06.579Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `query/query_report.rs`: `metric_summary` aggregates per unit; `MetricRow` (`:627-662`) fields are optional; `document_words` (`:1577-1583`) and `pages(row, words_per_page)` return `Option` and name their source; `share_value` (`:1567-1574`) and `ShareMetric` (`:602-623`): the share metric comes from the request (`CodeLines` only with `code`; `DocumentWords` only with `words`; otherwise the new `RawWords`).
- Absence is implemented once in the answer walks (P2.2.3 to P2.2.5) through `Presence::WhenAnalyzer`, not per writer; unrequested metrics have no key in JSON or YAML.
- Report schema `fdu.report/7`; per-unit coverage replaces a flat `coverage.analyzed`.
- Python `MetricValues` and `_metric_row` (`crates/fdu-py/python/fdu/_models.py:573-616`, `:924-948`) read optional fields (the full model move is P2.2.8).

**Tests**

- Goldens that run analysis (at least `cli-content`) lose unrequested metric keys.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: per-unit coverage maps break readers of a flat `coverage.analyzed`.
