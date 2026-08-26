---
type: is
id: is-01m0y1sjbfs5h264xhme2vqymg
title: Implement the thin MetaBrowser fdu backend and async change bridge
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sjnptgqhgvqcx1cjkkhw
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:34.671Z
updated_at: 2026-08-26T03:42:13.113Z
---
Add FduInventoryBackend and its handle on PR #74. Map the eight queries, config, paths, rows, state, work, and impacts without a second index. Run bounded operations with the existing asyncio.to_thread policy. Give each handle one dedicated change-poll worker, a one-slot locked mailbox, and an asyncio.Event woken through loop.call_soon_threadsafe; keep one result pending without advancing the cursor until consumption. Prove iterator-only cancellation, explicit optional packaging, typed unavailable errors, and no silent fallback.
