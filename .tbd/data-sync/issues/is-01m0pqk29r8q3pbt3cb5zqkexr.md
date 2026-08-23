---
type: is
id: is-01m0pqk29r8q3pbt3cb5zqkexr
title: No ledger job measures the default command, so default-path regressions are unmeasurable
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-23T07:15:34.840Z
updated_at: 2026-08-23T07:15:34.840Z
---
Found in the PR #38 senior review (https://github.com/jlevy/fdu/pull/38#issuecomment-5384769585).

0 of the 66 recorded artifacts measure "fdu <dir>" -- scan, index, rendered tree, snapshot write. The nearest proxy, cold-scan-index, is the probe walk plus index build and excludes rendering and the write entirely. The fdu-default-tree installed-command contract landed under fdu-ao6p and has never had an artifact recorded through it; the cache-layers plan already noted the same gap ("the harness has no job for the default one-shot CLI plan"), and its own decomposition put the snapshot write at ~36% of a default run on /usr -- work the proxy never times.

Both engine defects found by that review live in this blind spot.

Deliverable: a default-tree probe job with an oracle, recorded once now as a baseline on the nominated real subjects and re-recorded after every landing that touches the default path.
