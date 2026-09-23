---
type: is
id: is-01m36bkhqcnfts27ggfc5pwtfd
title: Reconcile the alpha plan checkboxes and core-model status with actual evidence
kind: task
status: open
priority: 1
version: 2
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:22.284Z
updated_at: 2026-09-23T05:25:51.434Z
---
Stack review R110-1..3. At #117 the alpha plan checks the local make check box on scoped runs although its Testing Strategy requires make check and forbids reading unfinished checks as passes; boxes are checked while beads remain open (fdu-0ssl, fdu-bwo2, fdu-5w7f, fdu-c22r, fdu-93e8 unsynced, fdu-6act, fdu-ns3o); the core-models plan Current Delivery still says the execution-plan model remains to be implemented; evidence is anchored at ff2b07da while later commits exist. Replace internal reviewer names (Astra, Sol) and machine-local worktree references. Do it in the top layer.
