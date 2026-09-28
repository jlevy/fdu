---
type: is
id: is-01m3kae7agsc4dtrx6jkffxfwx
title: Prototype an opt-in APFS gencount-gated walk in the engine
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:15:06.829Z
updated_at: 2026-09-28T06:15:06.829Z
---
Follow-up to fdu-gpqz (go with conditions). Read ATTR_CMNEXT_RECURSIVE_GENCOUNT in the existing getattrlistbulk walk; skip subtrees of origins whose gencount matches the checkpoint; treat 0 as descend; mark new directories on discovery (explicit opt-in, user-owned only; warn before marking large populated roots: 66 s synchronous at 225k entries with ~12x p90 I/O slowdown); never issue the totals fsctl on a non-origin; periodic full sweep; mtime/chmod/xattr-only changes are invisible. Unset is fsctl 0xC1104A71 with u32 +4 = 3 (undocumented; re-verify per OS). Go if: no oracle misses on the agent workload, refresh <= 20% of the walk at >= 1M entries, documented marking/unmarking contract. Evidence: explorations/change-sources/dirstats-verify/.
