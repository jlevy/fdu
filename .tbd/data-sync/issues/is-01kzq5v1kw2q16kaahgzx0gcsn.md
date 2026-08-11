---
type: is
id: is-01kzq5v1kw2q16kaahgzx0gcsn
title: "Spike: validate FSEvents cross-restart replay on a real volume"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq53dw7bawahp6m77sx9gd4
created_at: 2026-08-11T01:08:54.523Z
updated_at: 2026-08-11T05:38:43.750Z
closed_at: 2026-08-11T05:38:43.749Z
close_reason: "Spike complete; verdict GO with reduced expectations and a harder gate. CONFIRMED: deep in-place edit at depth 17 changed no directory mtime yet produced exactly one event naming the parent dir (the load-bearing claim); creates coalesce per directory (20 files+2 dirs = 7 events; 257-entry clone = 47 events, every dir named); fsevent-sys from the lockfile plus six self-declared externs links and drives the non-deprecated dispatch-queue path with zero new crates; volume UUID resolves from st_dev; HistoryDone sentinel path is meaningless as assumed. REFUTED 1: replay is not free - bimodal ~10ms/~200ms empty on a quiet tree over 17 trials, growing to 1.8s from a -20M-id cursor; the plan's 'tens of ms at 60k' was optimistic and Phase 2 acceptance moves to 500k+ or cold cache. REFUTED 2 (correctness): insufficient history is SILENT - replay from sinceWhen=1 returned HistoryDone after 1 event with no MustScanSubDirs, though the tree's clone would have logged ~10k events; a future cursor likewise returns empty and succeeds. Consequences landed in the spec (b639a83): G5 tightened 14d -> 24h and reclassified as the only protection, new G11 replay budget, new G12 mandatory periodic full sweep. Spike code was throwaway and is not committed."
---
Phase 0 of the FSEvents-scoped revalidation plan, promoted to first position because the research's source-level read showed cross-restart sinceWhen replay is unproven in production (Watchman uses resync only in-process, off by default, reverted Dec 2021 over possible correctness issues; git fsmonitor starts at SinceNow). Throwaway probe, not shipped code: dispatch-queue delivery (the non-deprecated API) end to end; strictly-greater-than sinceWhen; content edit deep in a large real tree (metabrowser-class clone) names the parent directory; deletions and renames; hour-old and week-old cursor replay latency; sinceWhen predating retention produces MustScanSubDirs rather than silent emptiness; FSEventsCopyUUIDForDevice stability across remount; permission/TCC surface; and above all cross-restart reliability - cursor written by one process, mutations made while no process exists, replay by a fresh process, and after reboot where practical. Any loss arriving WITHOUT a degradation flag changes the design (periodic full sweep promoted from paranoia to contract). Findings amend the spec's gates and constants; explicit go/no-go for Phase 2.
