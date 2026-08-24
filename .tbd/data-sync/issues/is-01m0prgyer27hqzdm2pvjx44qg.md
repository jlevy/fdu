---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Partitioned tallies: tag rules and per-plane roll-ups in the engine"
kind: feature
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:53.944Z
updated_at: 2026-08-24T00:53:40.827Z
---
Opt-in tag configuration on ScanOptions: compiled gitignore (ignore-crate matcher, correct negation) and hidden-with-allowlist rules; entry tag bits; per-plane roll-up state (files, dirs, bytes, allocated, newest mtime, per-extension) through merge_upward, refresh, and watch re-tagging; .gitignore-edit escalation to InvalidateSubtree; enabled rule set versions the snapshot fingerprint. Builds on the closed spike fdu-p35d (0.39-1.76 us/entry measured). Partition-sum property tests and fingerprint-invalidation tests land with it.

## Notes

SCOPE SETTLED — the earlier "awaiting metabrowser confirmation" note is stale and the
answer came back smaller than the bead's description.

The reconciliation (research-2026-08-23-interactive-contract-reconciliation.md) and
metabrowser's reply on PR #44 resolve it: hidden paths PRUNE AT SCOPE with an exact-name
allowlist, they do not become a second maintained tag plane. The plane had no consumer
once the product had no hidden toggle, and a tag plane would still have to walk the
hidden trees it exists to exclude. The third-plane open question closes as no.

So GITIGNORE IS THE SOLE TAG RULE here, and this bead is two pieces of work rather than
three: the gitignore matcher plus the unignored plane through the reducer path, and
hidden admission as a scope rule fingerprinted into snapshot identity.

WHAT ACTUALLY BLOCKS IT NOW: only the `ignore` crate, which is not yet a dependency and
owes the 14-day supply-chain cool-off. That is a scheduled step, not an open question --
the spike fdu-p35d already measured the matcher at 0.39-1.76 us/entry and closed.

READ BEFORE BUILDING: fdu-n4gn prices this plane together with groups, composed
provenance and leaf counts as ONE reducer union, on the ancestor-merge path exp-064 took
from 43.73% to 14.07% and campaign 2 plans to delete rather than tune. A cost acceptable
for each alone can be wrong in combination, and leaf counts have already shipped
unpriced (fdu-2ig2).
