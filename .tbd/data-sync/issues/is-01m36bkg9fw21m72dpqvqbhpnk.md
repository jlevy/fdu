---
type: is
id: is-01m36bkg9fw21m72dpqvqbhpnk
title: Snapshot admission is still decided in three places
kind: task
status: closed
priority: 2
version: 3
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:20.815Z
updated_at: 2026-09-23T08:07:00.103Z
closed_at: 2026-09-23T08:07:00.101Z
close_reason: "Fixed in f8367de5 on #115 (Plan::admit single decision); delta-reviewed; CI green"
resolution: null
duplicate_of: null
---
Stack review R115-3 (#115). execute admits the warm path inline (load_serving + root check); Plan::admit is consulted only on the cache-only branch (its fallback arm unreachable); persist_index_changes and run_facts re-derive root + serves_snapshot. They agree today via serves_snapshot, but fdu-xgjx acceptance says no route decides reads outside the plan. Fix: call Plan::admit on every Load::Snapshot and pass the relation into run_facts.
