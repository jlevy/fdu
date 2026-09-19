---
type: is
id: is-01m0py2a8eb90n6r21f4hygyvr
title: "Content-tier instance of H86: key roll-ups by EntryId and defer to one bottom-up pass"
kind: task
status: in_progress
priority: 1
version: 6
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
hold: null
hold_until: null
created_at: 2026-08-23T09:08:45.960Z
updated_at: 2026-09-19T06:23:19.676Z
started_at: 2026-09-19T06:07:37.656Z
---
The campaign plan's Phase C 'fdu-cq7t follow-on', which had no bead. H94 (exp-064/065) made ContentIndex::merge_ancestors cheap; this deletes it: key roll-ups by EntryId and compute them in one bottom-up pass, the shape that won -51.9% on snapshot load (4cc157d). Structural track: one composite experiment, differential oracle (content digest) plus pre-registered targets, measured on a dense real subject (cargo-registry-src is sparse-safe at 0.92 but only 5.8k entries here; exp-065's Linux subject was 13k). Plan against the warm number: content-cache-hit -25.78% was the transferable result.

## Notes

2026-09-19: H83's structural form remains queued behind a stage split. exp-108 put
load_content at 60.5% of the deciding-scale warm content engine; it did not say whether
that is parse, candidate install, or apply/merge. H112 / exp-109 measures that split
before this bead's EntryId + one-pass rewrite. Do not start the composite tonight.

2026-09-19 (exp-109 / H112): apply dominates restore (timers 63%, sample 54% of
load_content). Parse is about 10% — not this bead. The remaining structural
target is apply / commit / merge_ancestors, not a format-speed tweak.
H113 (duplicate completeness walk) is smaller than this rewrite and is next.
This bead stays open. Do not start the EntryId + one-pass composite until H113
is measured.
