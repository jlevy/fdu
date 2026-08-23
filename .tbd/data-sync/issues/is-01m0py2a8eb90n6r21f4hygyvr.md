---
type: is
id: is-01m0py2a8eb90n6r21f4hygyvr
title: "Content-tier instance of H86: key roll-ups by EntryId and defer to one bottom-up pass"
kind: task
status: open
priority: 1
version: 3
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-23T09:08:45.960Z
updated_at: 2026-08-23T09:09:10.808Z
---
The campaign plan's Phase C 'fdu-cq7t follow-on', which had no bead. H94 (exp-064/065) made ContentIndex::merge_ancestors cheap; this deletes it: key roll-ups by EntryId and compute them in one bottom-up pass, the shape that won -51.9% on snapshot load (4cc157d). Structural track: one composite experiment, differential oracle (content digest) plus pre-registered targets, measured on a dense real subject (cargo-registry-src is sparse-safe at 0.92 but only 5.8k entries here; exp-065's Linux subject was 13k). Plan against the warm number: content-cache-hit -25.78% was the transferable result.
