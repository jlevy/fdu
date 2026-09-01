---
type: is
id: is-01m1edc4xady6k86e0hsbzfsk1
title: Compact optional fixed-partition storage
kind: task
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T11:58:48.745Z
updated_at: 2026-09-01T12:16:03.637Z
started_at: 2026-09-01T11:58:57.120Z
closed_at: 2026-09-01T12:16:03.636Z
close_reason: "Rejected exp-084 after the pre-registered gate: exact compact storage removed 56 requested bytes per entry and improved default-tree 2.628% (95% CI -3.172% to -1.188%) with 18.159% lower RSS, but missed the required 3% wall threshold and cold-scan-index was inconclusive. The 260-line spike was reverted and recorded."
resolution: null
duplicate_of: null
---
Pre-registered H98 composite: replace the always-inline second InternedRollUp with optional storage allocated only for directories that maintain ignore partitions, and combine it with the exp-083 control-free maintenance lane. The post-exp-083 profile leaves about 90 requested bytes and 0.22 allocation/reallocation events per entry versus b75bf85, matching the oversized per-entry representation. Primary accept gate: metabrowser-current default-tree wall improves at least 3% versus 3c0e1a2 with the paired 95% interval below zero. Secondary gates: cold-scan-index moves in the same direction; exact digests and all control semantics match; the final allocation/reallocation/byte ratios against b75bf85 are each at most 1.05; opened-discovery has no greater than 3% wall/component upper bound and no unbounded allocation or RSS regression. Reject and revert the full composite if these gates fail.
