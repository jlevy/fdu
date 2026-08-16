---
type: is
id: is-01m044c6qj93t0p9bs6rjryyaz
title: Reserve experiment and hypothesis ids at registration time
kind: task
status: open
priority: 2
version: 1
labels:
  - performance
dependencies: []
created_at: 2026-08-16T01:53:27.280Z
updated_at: 2026-08-16T01:53:27.280Z
---
Two concurrent campaigns (the macOS adaptive-worker campaign on PR28's branch and the
Linux consumer review on PR29's) independently claimed experiment ids exp-056..059 for
eight different experiments, and used the H86/H87 numbers for unrelated hypotheses
(H86-observability / H87-fixed-worker-knee vs the arena epic / spawn_save clone).
Resolved during the PR29 re-stack by renumbering PR29's artifacts to exp-060..063 and
regenerating the ledger (64 artifacts validate; both campaigns coexist). The suffixed
H-labels remain distinct strings, so no data was corrupted, but the namespace is
confusing.

The ledger's design ("regenerated from artifacts, cannot drift") catches collisions
only at merge time. For the backfill-every-change discipline to hold with multiple
agents, id allocation needs to move to registration time. Options, cheapest first:

1. A CHECK: perf-ledger (or a pre-commit script) fails on duplicate experiment ids or
   on a bare H-number registered twice with different titles -- turns silent collision
   into a loud one. This much is nearly free and worth doing regardless.
2. A RESERVATION convention: hypotheses and experiment numbers are claimed by a
   one-line commit to the ledger's registry (or a bead) before the campaign runs,
   which serializes allocation through git history.
3. Namespaced ids (campaign prefix) if concurrent campaigns become the norm.

Also part of the same discipline: the session-scale measurements that currently live
only in beads (fdu-wpku's read-gate evidence, fdu-m4r6's macOS validation, fdu-f6n7's
dumac head-to-head) should be re-run through the harness once fdu-ao6p lands its
default-CLI job, and recorded as proper artifacts -- backfilling them into the ledger
with the same subjects and the evidence qualifiers PR28 added (PERF_STAGE,
PERF_HOST_REGIME), so the entire history stays consistent experiment by experiment.
