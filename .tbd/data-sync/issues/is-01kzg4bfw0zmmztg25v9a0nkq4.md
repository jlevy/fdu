---
type: is
id: is-01kzg4bfw0zmmztg25v9a0nkq4
title: "Watch hardening: rename stitching, backend selection, kqueue sweep, failed-watch marking"
kind: feature
status: open
priority: 2
version: 9
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - concurrency
  - watch
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:15.231Z
updated_at: 2026-08-09T22:08:33.418Z
---
Harden the platform-specific watch semantics after the transport and lifecycle contract is made safe under fdu-8jte. Scope here is backend behavior, not generic queue ownership: pair inotify rename cookies without unnecessary filesystem access; stitch renames elsewhere by stable file identity and escalate unpaired cases; select native backends for local filesystems and polling for NFS/FUSE/CIFS; add periodic reconciliation where kqueue cannot signal overflow; mark failed watch coverage as non-fresh until recovery; cap kqueue descriptor usage and degrade explicitly; and limit new-directory relisting to backends with a registration race. Every backend-specific loss or ambiguity becomes InvalidateSubtree and closes through reconciliation. Acceptance covers supported OS backends and simulated overflow/rename/install failures without weakening the bounded, nonblocking, cancellable worker protocol owned by fdu-8jte.

## Notes

Generic transport/lifecycle prerequisites fdu-s7wr and fdu-8jte are complete: bounded nonblocking ingress/output, capped coalescing, sticky overflow invalidation, consuming-thread verification, cancellation/join, and typed stop/panic are established. This bead remains limited to backend-specific rename stitching, selection, descriptor/coverage failure, and periodic reconciliation behavior.
