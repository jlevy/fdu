---
type: is
id: is-01m4p4a90e3qdm7vyv7any3tj3
title: Re-record the 0.5.0 demo on a quiet host with real ages and the new colors
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-11T18:41:31.145Z
updated_at: 2026-10-11T18:42:03.770Z
---
The kernel tree at ~/fdu-demo-trees/linux now carries each entry's last commit time (packages/cli-animate/examples/fdu/commit-mtimes.py, history in a blobless clone on the external drive). A color preview recorded 2026-10-11 11:13 at load average ~91 ran 4-10x slow (tree 1.43 s, first analysis 36.9 s, cached 7.7 s, against 0.39/8.1/0.75 s the night before) and must not ship. Re-record from the release commit's build when the host is quiet (check uptime first; stop other sessions' builds), compare receipt timings with the v0.4.0 binary in fdu-v0.4.0-diag, then put the GIF and the MP4's size and sha256 in docs/media in the release PR. Ages are measured from now, so record soon after any re-dating.
