---
type: is
id: is-01kzg4d256qmchmtyvttnpvn4y
title: "Content-tier metrics: line, word, sentence, paragraph counts"
kind: feature
status: closed
priority: 3
version: 9
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies: []
parent_id: is-01kzm3v6nndedpwk414enwysv3
child_order_hints:
  - is-01kzwtrsf5bjtwza5q5ayp014g
  - is-01kzwxej9axvrpknyke9hz9m2k
  - is-01kzx1aabeghy62dfp0gk03fbr
created_at: 2026-08-08T07:29:06.726Z
updated_at: 2026-08-13T12:03:10.093Z
closed_at: 2026-08-13T12:03:10.093Z
close_reason: "All six content-metrics phases are implemented and validated by the complete local gate and green cross-platform PR #10 checks; the plan and documentation are reconciled."
---
Deferred past phase 1 deliberately: build it once the stat tier is solid. Its PLACE is reserved now — in the reducer registry and in the per-analyzer fingerprint cache — because that shapes the snapshot format and cannot be retrofitted cheaply.

The finding that makes this worth building: scc and tokei are state of the art for per-file content metrics at scale, and NEITHER caches anything across runs, NEITHER supports per-directory roll-up. Both aggregate per-language, globally. The two things most needed from a content-metric layer are unbuilt in the best-in-class tools.

Techniques when it is time:
- Byte-mask pre-filter to skip uninteresting bytes branchlessly (scc: currentByte & mask == currentByte).
- Aho-Corasick prefilter to find the first interesting byte, with a cheap parallel pass over the boring prefix (tokei).
- Reuse a per-worker read buffer, discarding it if it grows past a threshold.
- Cache by (stat fingerprint, analyzer id, analyzer version) — flowmark's fingerprint invalidation applied per-analyzer rather than whole-cache, so adding an analyzer does not invalidate the others.
- Content work must be opt-in and lazy. Every system that touches content bounds the read or caches by identity.
