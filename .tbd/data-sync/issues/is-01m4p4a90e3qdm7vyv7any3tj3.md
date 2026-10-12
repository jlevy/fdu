---
type: is
id: is-01m4p4a90e3qdm7vyv7any3tj3
title: Re-record the 0.5.0 demo on a quiet host with real ages and the new colors
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-11T18:41:31.145Z
updated_at: 2026-10-12T07:14:54.620Z
started_at: 2026-10-12T06:54:32.258Z
---
The kernel tree at ~/fdu-demo-trees/linux now carries each entry's last commit time (packages/cli-animate/examples/fdu/commit-mtimes.py, history in a blobless clone on the external drive). A color preview recorded 2026-10-11 11:13 at load average ~91 ran 4-10x slow (tree 1.43 s, first analysis 36.9 s, cached 7.7 s, against 0.39/8.1/0.75 s the night before) and must not ship. Re-record from the release commit's build when the host is quiet (check uptime first; stop other sessions' builds), compare receipt timings with the v0.4.0 binary in fdu-v0.4.0-diag, then put the GIF and the MP4's size and sha256 in docs/media in the release PR. Ages are measured from now, so record soon after any re-dating.

## Notes

Completed 24 matched exploratory samples (old/current release binaries, internal/external content caches, three trials) plus actual PTY checks. All stdout identical, all exits zero, all95938 fresh/cached counts correct. Internal medians old8.868/0.770s and current9.204/0.784s; external-cache medians old8.894/0.827 and current9.170/0.768. PTY old8.918/0.765 and current9.354/0.783. This approximately recovers the earlier performance range and does not reproduce the historical multi-fold slowdown. All matched CPU boundaries exceeded25 percent: exploratory, not a small-regression verdict. Independent review confirmed bounded conclusions. Historical approved30.459/4.028 take was real; original capture host load and exact binary identity are unavailable. Nearby build finished before capture; prior renderer was signalled45s before capture but descendant shutdown unverified. Do not assert CPU contention or external cache as established cause. Diagnosis retained in durable release evidence/demo-diagnosis plus local visualization demo-performance-conclusion.json. Fresh no-pause demo capture now being prepared with timed binary/cache/source internal, conditions recorded; public README remains unchanged.
