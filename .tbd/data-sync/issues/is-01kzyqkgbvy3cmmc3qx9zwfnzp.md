---
type: is
id: is-01kzyqkgbvy3cmmc3qx9zwfnzp
title: Represent content coverage and cached results independently per analyzer
kind: bug
status: closed
priority: 0
version: 8
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj058tjrqdmdexgt0r88q6
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2phzn814exmf4ty5vw6zha0
hold: null
hold_until: null
created_at: 2026-08-13T23:34:02.872Z
updated_at: 2026-09-23T08:14:06.753Z
started_at: 2026-09-20T04:39:46.901Z
closed_at: 2026-09-23T08:14:06.753Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Profiles are documented as additive analyzer bundles, but FileAnalysis stores one profile-wide coverage outcome and MetricValues record. If code-sloc-v1 is unsupported, a code/full record discards content-basic-v1 metrics that did succeed. Align storage, rollups, reports, and sidecars with per-analyzer coverage and reuse.
