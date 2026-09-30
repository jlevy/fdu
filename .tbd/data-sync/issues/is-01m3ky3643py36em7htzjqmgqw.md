---
type: is
id: is-01m3ky3643py36em7htzjqmgqw
title: "timeline.py: recognize balanced-1m synthetic subject provenance"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-28T11:58:36.674Z
updated_at: 2026-09-30T02:56:04.755Z
closed_at: 2026-09-30T02:56:04.755Z
close_reason: "Already fixed by 551cf044 (2026-09-29, in this branch): is_synthetic checks every generator in TREE_GENERATORS, including benchmarks.generate, whose balanced recipe built the million-entry tree. Verified: all three 1,000,001-entry subjects in docs/project/reports/performance-evidence/timeline.json (two Linux, one macOS) have synthetic: true; test_timeline covers it; make perf-report-check passes."
resolution: null
duplicate_of: null
---
Flag from the 2026-09-28 macOS rerun (PR #147): the timeline's synthetic-subject check does not recognise balanced-1m provenance, so macOS and Linux runs on that generated tree are marked as real trees. Fix in the benchmark timeline tooling (timeline.py).
