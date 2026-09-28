---
type: is
id: is-01m3k8bv5dqnr0phk1a24we46n
title: "Experiment: APFS dir-stats pruned refresh at scale vs walk oracle"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T05:38:51.691Z
updated_at: 2026-09-28T06:15:09.985Z
closed_at: 2026-09-28T06:14:40.327Z
close_reason: "Verified 2026-09-27 (explorations/change-sources/dirstats-verify/). Signal: go with conditions — unprivileged marking works on internal and external volumes; gencount updates synchronously on every size/membership change incl. open-writer write(); pruned refresh at 225,653 entries 1.04 s vs 7.9-9.7 s walk with 0 size/membership misses (mtime-only touches missed by design); persistence and forced-detach safe, fsck clean; undocumented unset (+4=3). Costs: marking a populated 225k root is a 66 s synchronous fsctl degrading concurrent I/O ~12x at p90; origins at every level add 26-86% write cost. Totals: no-go as authoritative (primary-link rule, no rsrc/xattr, open-writer lag up to 11 s, drift reports on real volumes). Follow-ups: , ."
resolution: null
duplicate_of: null
---
Rank 1 in the review. Go if: zero oracle misses across randomized workloads incl. open writers; refresh <=10% of walk at >=200k entries; write overhead <=10% at realistic depth; clean removal path; works unprivileged on the internal volume. Private marking API (apfs.util -M / fsctl 0xC1104A71) and persistent inherited flag are the risks; owned fixtures and disk images only.

## Notes

Follow-ups (the close reason's list was lost to a scripting error): fdu-ns3n (opt-in gencount-gated walk in the engine), fdu-22hd (in-kernel sizing of unmarked directories).
