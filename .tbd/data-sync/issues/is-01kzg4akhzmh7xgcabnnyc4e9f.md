---
type: is
id: is-01kzg4akhzmh7xgcabnnyc4e9f
title: Hardlink attribution policy that survives incremental updates
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:46.239Z
updated_at: 2026-08-08T07:27:46.239Z
---
No prior art to copy — this needs design, not just a choice.

The surveyed tools all differ: dust uses an order-dependent global seen-set, dua counts down remaining links, gdu divides size among linked items, dut tracks shared-vs-unique in two columns, ncdu uses circular linked lists per inode group with a hash map keyed on (dev, ino).

dut's shared/unique split is the most informative for a user. But for STABLE, CACHEABLE roll-ups the rule must be deterministic and it must survive incremental updates — and none of these tools attempt that, because none of them revalidate. Decide the rule, then prove it holds under delta application (add a link, remove a link, remove the last link).

ncdu's uncounted-set trick is worth borrowing: fall back to full iteration once the set exceeds one eighth of the map, so work is bounded either way.
