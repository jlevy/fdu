---
type: is
id: is-01m4890pwshjnenvprkn7gh3w5
title: "PR #177 C4: RequestError is #[non_exhaustive] but its variants are not; this release added fields to ViewNeedsAnalyzer, SortNeedsAnalyzer and WatchContent. Mark those variants #[non_exhaustive] (a breaking change"
kind: task
status: in_progress
priority: 3
version: 4
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: null
hold: null
hold_until: null
created_at: 2026-10-06T09:34:18.519Z
updated_at: 2026-10-06T15:58:17.298Z
started_at: 2026-10-06T15:58:17.297Z
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. RequestError is #[non_exhaustive] but its variants are not; this release added fields to ViewNeedsAnalyzer, SortNeedsAnalyzer and WatchContent. Mark those variants #[non_exhaustive] (a breaking change: do it with the next minor).

## Notes

Not a patch item: breaking; schedule with the next minor (0.5.0).
