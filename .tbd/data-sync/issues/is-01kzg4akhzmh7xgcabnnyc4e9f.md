---
type: is
id: is-01kzg4akhzmh7xgcabnnyc4e9f
title: Hardlink attribution policy that survives incremental updates
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - design-gate
dependencies:
  - type: blocks
    target: is-01kzg49sswr78gpjykxctbe6c7
  - type: blocks
    target: is-01kzg4ajxc0pvgcmj834gahcgt
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:46.239Z
updated_at: 2026-09-27T06:51:13.298Z
---
No prior art to copy — this needs design, not just a choice.

The surveyed tools all differ: dust uses an order-dependent global seen-set, dua counts down remaining links, gdu divides size among linked items, dut tracks shared-vs-unique in two columns, ncdu uses circular linked lists per inode group with a hash map keyed on (dev, ino).

dut's shared/unique split is the most informative for a user. But for STABLE, CACHEABLE roll-ups the rule must be deterministic and it must survive incremental updates — and none of these tools attempt that, because none of them revalidate. Decide the rule, then prove it holds under delta application (add a link, remove a link, remove the last link).

ncdu's uncounted-set trick is worth borrowing: fall back to full iteration once the set exceeds one eighth of the map, so work is bounded either way.

## Notes

2026-09-14 (PR #55 review, 4727de0): the disk-usage checkpoint plan's recommended default delta measure, unique allocated bytes counted once per (dev, inode) within a checkpoint, depends on this rule. Its slice 2 (fdu-8ybz) uses a provisional attribution, the in-scope path that sorts first by bytes, which is deterministic only across complete captures. This bead's rule has to replace it before incremental refresh (slice 3). The engine retains no link count today (crates/fdu-core/src/engine_contract.rs:81-95 at dda7e6a); the plan proposes retaining one so only multiply-linked files are grouped.

2026-09-26 inventory audit (fdu-6u68): current per-path accounting was confirmed with two hard-linked .venv files sharing one (device,inode): each reports 16,384 allocated bytes, Summary 32,768. Matching-root union is already deduplicated but file identities are not. Retain this bead as the design owner; no duplicate feature created. New assessment: docs/project/specs/active/plan-2026-09-26-directory-inventory-accounting.md. A future unique-identity measure must not imply clone-extent deduplication or reclaimable bytes; Windows currently lacks observed allocation/identity for that claim. Current uv defaults are clone on macOS/Linux, hardlink on Windows; mode and cross-filesystem copy fallback matter.
