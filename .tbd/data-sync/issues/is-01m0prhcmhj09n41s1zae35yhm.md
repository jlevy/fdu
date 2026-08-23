---
type: is
id: is-01m0prhcmhj09n41s1zae35yhm
title: "Session integration shape: mid-walk progress, async form, session-to-watch clock handoff"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.465Z
updated_at: 2026-08-23T08:20:37.908Z
---
Three requirements that land with the progressive-results session, not after it: progress readable mid-walk (entries applied, clock, completeness) for crawl-status UIs; the async shape shipping with the sync one (same adapter policy as watch); and the walk-complete clock being the clock a watch resumes from, tested for the no-gap property.

## Notes

The session is the same pull pattern as the watch session, pointed at the scan producer instead of the watch producer — scan.rs and watch.rs are already both metadata-delta producers, so this needs a window onto the existing stream rather than new machinery. Both producers mint the same delta type, so a client sees one stream shape for boot fill and for live changes; metabrowser already converged on that independently (its walker and watcher both emit FsChange(ops=(FsUpsert...)) over one SSE channel). Metabrowser's placeholder-then-finalized two-phase yield maps onto one delta whose roll-up grows through merge_upward with per-path status moving Partial to Complete.
