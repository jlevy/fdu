---
type: is
id: is-01m2eefw0awdfhkmw5sz1ax7k3
title: "PR #52 review BUILD-3: alternating fdu DIR and fdu --watch DIR overwrite each other's snapshot"
kind: task
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:01.097Z
updated_at: 2026-09-13T22:53:03.312Z
closed_at: 2026-09-13T22:52:58.059Z
close_reason: "Documented in the PR #52 commit that follows b82a0e5: CachePolicy::Auto rustdoc and the README's usable-cache paragraph state that fdu <dir> and fdu --watch <dir> keep snapshots of different scope at one cache path and replace each other's, naming the runs that start cold. Wording follows #51's report/open note; keyed snapshots not pursued."
resolution: null
duplicate_of: null
---
PR #52 review BUILD-3 (Low). crates/fdu-core/src/lib.rs:334-352 and 377-395 at afbb2ee. snapshot_scope_serves correctly treats a controls-on snapshot as absent for a scanning controls-off request, but under CachePolicy::Auto that request's save overwrites the watch-written snapshot, and the next fdu --watch misses in turn. Correct, and visible as ReportSource::ColdScan, but it changes the cost model for anyone alternating modes. Fix: document it where the CLI or cache policy is documented (chosen), consistent with #51's wording for the report/open split, or key snapshots by scope.

## Notes

Fixed in e3b5103 on PR #52 (CachePolicy::Auto rustdoc and README).
