---
type: is
id: is-01kzq5v1kw2q16kaahgzx0gcsn
title: "Spike: validate FSEvents cross-restart replay on a real volume"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53dw7bawahp6m77sx9gd4
created_at: 2026-08-11T01:08:54.523Z
updated_at: 2026-08-11T05:32:40.826Z
---
Phase 0 of the FSEvents-scoped revalidation plan, promoted to first position because the research's source-level read showed cross-restart sinceWhen replay is unproven in production (Watchman uses resync only in-process, off by default, reverted Dec 2021 over possible correctness issues; git fsmonitor starts at SinceNow). Throwaway probe, not shipped code: dispatch-queue delivery (the non-deprecated API) end to end; strictly-greater-than sinceWhen; content edit deep in a large real tree (metabrowser-class clone) names the parent directory; deletions and renames; hour-old and week-old cursor replay latency; sinceWhen predating retention produces MustScanSubDirs rather than silent emptiness; FSEventsCopyUUIDForDevice stability across remount; permission/TCC surface; and above all cross-restart reliability - cursor written by one process, mutations made while no process exists, replay by a fresh process, and after reboot where practical. Any loss arriving WITHOUT a degradation flag changes the design (periodic full sweep promoted from paranoia to contract). Findings amend the spec's gates and constants; explicit go/no-go for Phase 2.
