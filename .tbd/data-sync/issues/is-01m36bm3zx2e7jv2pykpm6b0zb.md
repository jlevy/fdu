---
type: is
id: is-01m36bm3zx2e7jv2pykpm6b0zb
title: Fold [Unreleased] into 0.1.0 and bring the release notes up to the composed candidate
kind: task
status: open
priority: 1
version: 2
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:40.979Z
updated_at: 2026-09-23T05:25:50.810Z
---
At #117 CHANGELOG has an [Unreleased] section above an unpublished [0.1.0] (placeholder date 2026-09-16), and docs/project/release-notes/0.1.0.md does not mention directory filters, the tree/paths/long presentations, the machine default switching to list, Python refresh/watch persistence, or the fdu.report/7, fdu.stream/2, fdu.cache/2 schemas as final. Nothing has ever been published as 0.1.0, so fold Unreleased into 0.1.0 at tag time and set the date. Include fdu-twry (VALIDITY_VERSION bump strands every store). Do it on the release-prep commit after the stack and perf layers merge.
