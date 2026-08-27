---
type: is
id: is-01m0y1sjbfs5h264xhme2vqymg
title: Implement the thin MetaBrowser fdu backend and async change bridge
kind: feature
status: open
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sjnptgqhgvqcx1cjkkhw
  - type: blocks
    target: is-01m10nsf27ydw4sb116neghkg1
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nsdjx7z9h87m4nf8hzhyh
  - is-01m10nse28b6m4gajwbks6a08g
  - is-01m10nsed829d5v3qndj1a2hg7
  - is-01m10nseqmb4w4n271gyre0xnp
created_at: 2026-08-26T03:28:34.671Z
updated_at: 2026-08-27T03:57:21.830Z
---
Add FduInventoryBackend and its handle on PR #74. Map the eight queries, config, paths, rows, state, work, and impacts without a second index. Run bounded operations with the existing asyncio.to_thread policy. Give each handle one dedicated change-poll worker, a one-slot locked mailbox, and an asyncio.Event woken through loop.call_soon_threadsafe; keep one result pending without advancing the cursor until consumption. Prove iterator-only cancellation, explicit optional packaging, typed unavailable errors, and no silent fallback.
