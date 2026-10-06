---
type: is
id: is-01m4890pwshjnenvprkn7gh3w5
title: "PR #177 C4: RequestError is #[non_exhaustive] but its variants are not; this release added fields to ViewNeedsAnalyzer, SortNeedsAnalyzer and WatchContent. Mark those variants #[non_exhaustive] (a breaking change"
kind: task
status: closed
priority: 3
version: 6
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-06T09:34:18.519Z
updated_at: 2026-10-06T16:32:53.188Z
started_at: 2026-10-06T15:58:17.297Z
closed_at: 2026-10-06T16:32:53.187Z
close_reason: "Fixed for 0.4.0 in 9c70561e (branch fix-177-D): the four variants are #[non_exhaustive]; PR #177 review D1."
resolution: null
duplicate_of: null
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. RequestError is #[non_exhaustive] but its variants are not; this release added fields to ViewNeedsAnalyzer, SortNeedsAnalyzer and WatchContent. Mark those variants #[non_exhaustive] (a breaking change: do it with the next minor).

## Notes

Done for 0.4.0, not 0.5.0 (PR #177 review D1, https://github.com/jlevy/fdu/pull/177#issuecomment-6020135457): 0.4.0 starts a new minor series and already introduces or reshapes every variant involved, so the attribute breaks nothing 0.4.0 does not already break, while deferring it would break the same variants again in 0.5.0. ViewNeedsAnalyzer, SortNeedsAnalyzer, WatchContent, and AnalyzerNamedAsView are #[non_exhaustive] in commit 9c70561e on branch fix-177-D (off main 55d66863); the CHANGELOG breaking entry says to match them with '..'. No surface needed a change: the only outside match (crates/fdu-core/tests/watch_session_integration.rs) already used '..', and the CLI and Python binding render via message(). make check passed.
