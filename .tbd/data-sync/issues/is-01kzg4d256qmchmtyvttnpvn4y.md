---
type: is
id: is-01kzg4d256qmchmtyvttnpvn4y
title: "Content-tier metrics: line, word, sentence, paragraph counts"
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-08T07:29:06.726Z
updated_at: 2026-08-08T07:29:06.726Z
---
Deferred past phase 1 deliberately: build it once the stat tier is solid. Its PLACE is reserved now — in the reducer registry and in the per-analyzer fingerprint cache — because that shapes the snapshot format and cannot be retrofitted cheaply.

The finding that makes this worth building: scc and tokei are state of the art for per-file content metrics at scale, and NEITHER caches anything across runs, NEITHER supports per-directory roll-up. Both aggregate per-language, globally. The two things most needed from a content-metric layer are unbuilt in the best-in-class tools.

Techniques when it is time:
- Byte-mask pre-filter to skip uninteresting bytes branchlessly (scc: currentByte & mask == currentByte).
- Aho-Corasick prefilter to find the first interesting byte, with a cheap parallel pass over the boring prefix (tokei).
- Reuse a per-worker read buffer, discarding it if it grows past a threshold.
- Cache by (stat fingerprint, analyzer id, analyzer version) — flowmark's fingerprint invalidation applied per-analyzer rather than whole-cache, so adding an analyzer does not invalidate the others.
- Content work must be opt-in and lazy. Every system that touches content bounds the read or caches by identity.
