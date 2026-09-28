---
type: is
id: is-01m3jmvw2cm5wwa1jwc5m80vsw
title: Use one elapsed-time format for progress and performance
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-27T23:58:05.388Z
updated_at: 2026-09-28T00:27:50.685Z
closed_at: 2026-09-28T00:27:50.685Z
close_reason: "Implemented and reviewed in 593cea57; all 19 implementation CI jobs pass. Full local handoff plus corrective Rust/library-only reruns pass. Fresh top build 22e69f53a installed with matching skill and 102/102 independent checks across 25 invocations. Review and evidence are recorded in PR #136 and the tally arithmetic review. Synthetic aggregate overflow remains separately tracked in fdu-sqyk."
resolution: null
duplicate_of: null
---

## Notes

Latest user instruction supersedes minute/hour formatting: share exact perf duration formatter; long scans display151.33 s, withfractionalprecision retained.
