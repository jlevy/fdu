---
type: is
id: is-01kzyqkgbvy3cmmc3qx9zwfnzp
title: Represent content coverage and cached results independently per analyzer
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies:
  - type: blocks
    target: is-01m2pj058tjrqdmdexgt0r88q6
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-08-13T23:34:02.872Z
updated_at: 2026-09-17T02:09:16.057Z
---
Profiles are documented as additive analyzer bundles, but FileAnalysis stores one profile-wide coverage outcome and MetricValues record. If code-sloc-v1 is unsupported, a code/full record discards content-basic-v1 metrics that did succeed. Align storage, rollups, reports, and sidecars with per-analyzer coverage and reuse.
