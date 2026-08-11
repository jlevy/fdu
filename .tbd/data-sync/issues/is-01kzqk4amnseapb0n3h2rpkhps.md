---
type: is
id: is-01kzqk4amnseapb0n3h2rpkhps
title: "PR #3 review R10: remove quadratic snapshot child lookup"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:10.164Z
updated_at: 2026-08-11T05:12:14.815Z
closed_at: 2026-08-11T05:12:14.814Z
close_reason: Fixed snapshot fanout by adding direct B-tree child lookup after Delta application, correcting the preorder memo contract, and adding a 4,096-sibling snapshot round-trip regression.
---
FDU-PR3-R10. crates/fdu/src/snapshot.rs parent resolution rescans children after every insertion, making a wide directory quadratic; the adjacent grouped-by-parent comment is false for preorder records. Add direct child lookup or return EntryId through the Delta boundary, correct the explanation, and cover wide fanout load scaling. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
