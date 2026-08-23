---
type: is
id: is-01m0p2vmh9q50tw47y0es6f5a0
title: Peer walker ranking needs several real trees before it can be claimed in either direction
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/reports/report-2026-08-23-metadata-walk-floor.md
labels: []
dependencies: []
created_at: 2026-08-23T01:13:15.561Z
updated_at: 2026-08-23T01:13:15.561Z
---
The Linux walker comparison inverts with the subject: fdu leads ignore (ripgrep's walker)
by 12-26% on four generated trees of different shapes, ties on a tree carrying real
filenames, and trails by about 12% on /usr. /usr is the only real tree measured, so the
sweep is enough to retire the generated-corpus ranking and not enough to establish the
real-tree one.

This is a second and orthogonal blocker on peer claims alongside fdu-ow8y, which is about
host control. That one says a quiet host is required; this one says a representative
corpus is, and the corpus effect measured here is larger than the host effect.

To close: several real trees of different character (a source checkout with node_modules,
a package cache, a media tree, a system prefix), on more than one host, with the pinned
binaries and installation attestation the macOS live tool comparison carries. Until then
the report states the inversion and claims no ordering.
