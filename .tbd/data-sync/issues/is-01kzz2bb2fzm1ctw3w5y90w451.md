---
type: is
id: is-01kzz2bb2fzm1ctw3w5y90w451
title: Decide what to do about ExtId and RollUp.by_ext crossing the public API boundary
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:49.647Z
updated_at: 2026-08-14T02:41:49.647Z
---
crates/fdu/src/index.rs exports pub type ExtId = u32 and pub by_ext: BTreeMap<ExtId, ExtTally> on RollUp. The ids are index-private interner slots, so a public roll-up is only interpretable by asking the same Index that produced it via by_ext_named. Comparing two indexes' by_ext maps, or holding a roll-up past a mutation, is meaningless in a way the type system does not prevent.

PR #4 split the type: an index-private InternedRollUp for the hot merge path and a public RollUp whose by_ext is keyed by String, materialized once at each query boundary. That keeps exp-008's accepted interning win where it matters while making the public type self-describing.

Every in-tree caller on main currently goes through by_ext_named and is correct, so this is a latent API hazard rather than a live bug, and the refactor reaches index.rs, scan.rs, query/query_report.rs, cli.rs, and fdu-py. Recorded as a deliberate decision for the maintainer rather than ported in the same change as the defect fixes.
