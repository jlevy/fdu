---
type: is
id: is-01kzyqkgbvy3cmmc3qx9zwfnzp
title: Represent content coverage and cached results independently per analyzer
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:34:02.872Z
updated_at: 2026-08-13T23:34:02.872Z
---
Profiles are documented as additive analyzer bundles, but FileAnalysis stores one profile-wide coverage outcome and MetricValues record. If code-sloc-v1 is unsupported, a code/full record discards content-basic-v1 metrics that did succeed. Align storage, rollups, reports, and sidecars with per-analyzer coverage and reuse.
