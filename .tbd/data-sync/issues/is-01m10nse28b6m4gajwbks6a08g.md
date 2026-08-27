---
type: is
id: is-01m10nse28b6m4gajwbks6a08g
title: Implement and prove the bounded MetaBrowser async change bridge
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nsed829d5v3qndj1a2hg7
parent_id: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-27T03:56:30.663Z
updated_at: 2026-08-27T03:56:31.016Z
---
Give each fdu handle one dedicated change-poll worker, one-slot locked mailbox, asyncio.Event wakeup, and at most one active async iterator. Hold a pending native result without advancing the local cursor until consumption; iterator aclose joins only the bridge within one poll interval and preserves handle reads; handle close joins bridge then native owner; prove backpressure reset and event-loop closure.
