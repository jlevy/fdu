---
type: is
id: is-01m0pqk29r8q3pbt3cb5zqkexr
title: No ledger job measures the default command, so default-path regressions are unmeasurable
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
  - macos-agenda
dependencies:
  - type: blocks
    target: is-01m0pqk1nrzyrd883jtjqf06y7
  - type: blocks
    target: is-01m0pqk1zqhx9tbhjz436n4pse
created_at: 2026-08-23T07:15:34.840Z
updated_at: 2026-08-23T09:35:25.408Z
closed_at: 2026-08-23T09:35:25.407Z
close_reason: "Landed: perf_probe default-tree mode + default-tree-first/default-tree jobs (tallies oracle, snapshot_written flag). exp-066 baseline on rustup-toolchains (175k): default-tree wall 362-387 ms, cold-scan-index component 307 ms, snapshot rewritten on 24/24 repeated trials. Commit on claude/research-loop-overnight-2026-08-23."
---
Found in the PR #38 senior review (https://github.com/jlevy/fdu/pull/38#issuecomment-5384769585).

0 of the 66 recorded artifacts measure "fdu <dir>" -- scan, index, rendered tree, snapshot write. The nearest proxy, cold-scan-index, is the probe walk plus index build and excludes rendering and the write entirely. The fdu-default-tree installed-command contract landed under fdu-ao6p and has never had an artifact recorded through it; the cache-layers plan already noted the same gap ("the harness has no job for the default one-shot CLI plan"), and its own decomposition put the snapshot write at ~36% of a default run on /usr -- work the proxy never times.

Both engine defects found by that review live in this blind spot.

Deliverable: a default-tree probe job with an oracle, recorded once now as a baseline on the nominated real subjects and re-recorded after every landing that touches the default path.
