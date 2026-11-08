---
type: is
id: is-01m6ycagweqhppqznjzwagp365
title: "Say what a filtered report admitted: selected vs scanned files and bytes"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
child_order_hints:
  - is-01m6yn4418zpvm5mpxnfdb3cds
  - is-01m6yn44yyd0z9gz709ag6hyks
created_at: 2026-11-08T20:06:46.926Z
updated_at: 2026-11-08T22:40:35.293Z
---
Phase 1. Every report whose selection narrows the scanned set ends with one note, e.g. 'note: selected 2,951 of 27,060 files, 704 MiB of 1.8 GiB', from the unfiltered root roll-up (free). Machine output carries selected and scanned tallies on the envelope (pending the open question). Today a filtered tree's root row reads 100% of the admitted total and a filtered summary prints only the admitted size, so a day's writes can be misread as the tree. Review every changed golden.
