---
type: is
id: is-01kzwet2bamkz0gwrs1p0dcv5b
title: Benchmark fdu against fast live disk-usage alternatives
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
dependencies: []
parent_id: is-01kzw3t92p7d4512h8vn6ktch1
created_at: 2026-08-13T02:21:51.849Z
updated_at: 2026-08-13T05:51:08.545Z
closed_at: 2026-08-13T05:51:08.544Z
close_reason: "Completed 12 paired trials per competitor on 976,295 entries: FDU beat every rendered/indexed alternative and all scalar alternatives except dumac; exact versions, hashes, commands, uncertainty, resources, and caveats are published."
---
Run interleaved, repeated, semantically comparable full-tree traversals on the same 100K+ local-SSD tree against representative fast maintained alternatives; record versions, commands, machine, tree identity, uncertainty, resource use, and semantic caveats.

## Notes

Exploratory exp041 on 1,010,866 entries: fdu 4.778s median; dust 8.048, gdu 8.822, pdu 7.369, dua 6.989, diskus 7.110, BSD du 18.746, GNU du 27.159, ncdu 26.944. Zero invalid samples and no mutation during the 240 traversals, but external baseline digest drifted before timing because a repository operation occurred between baseline and run. Do not publish. Fix workflow, commit source-derived queue, then rerun fast peers with contiguous baseline+measurement.
