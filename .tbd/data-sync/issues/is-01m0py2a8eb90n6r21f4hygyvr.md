---
type: is
id: is-01m0py2a8eb90n6r21f4hygyvr
title: "Content-tier instance of H86: key roll-ups by EntryId and defer to one bottom-up pass"
kind: task
status: in_progress
priority: 1
version: 11
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
child_order_hints:
  - is-01m2w71cmphbn0k2pk3w9xytk8
  - is-01m2w7kkps03c1thq1p87mpm4p
hold: null
hold_until: null
created_at: 2026-08-23T09:08:45.960Z
updated_at: 2026-09-19T07:25:31.766Z
started_at: 2026-09-19T06:07:37.656Z
---
The campaign plan's Phase C 'fdu-cq7t follow-on', which had no bead. H94 (exp-064/065) made ContentIndex::merge_ancestors cheap; this deletes it: key roll-ups by EntryId and compute them in one bottom-up pass, the shape that won -51.9% on snapshot load (4cc157d). Structural track: one composite experiment, differential oracle (content digest) plus pre-registered targets, measured on a dense real subject (cargo-registry-src is sparse-safe at 0.92 but only 5.8k entries here; exp-065's Linux subject was 13k). Plan against the warm number: content-cache-hit -25.78% was the transferable result.

## Notes

2026-09-19 exp-112 / H115 accepted. Restore-only bottom-up roll-up after
sidecar inserts: wall -9.69% [-26.02%, -7.13%]. User CPU -8.23%.
Engine kept (7798fdc1). Child fdu-wx15 closed.

This bead stays open as the EntryId composite parent. Do not restart that
rewrite from the H115 accept. Do not retry H114 or H115.

2026-09-19 exp-111 / H114 rejected. Type-id String alloc on ContentRollUp::add
is not the remaining restore win: wall -0.56% [-17.92%, +4.79%]. Engine
reverted.
