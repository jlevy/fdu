---
type: is
id: is-01m19txpnb1qrgdt42kmmp01cg
title: MetaBrowser branch chain cannot reach main as it stands
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-30T17:19:23.306Z
updated_at: 2026-08-30T17:19:23.306Z
---
The paired MetaBrowser work is a three-layer stack, and the middle layer has no pull
request, so nothing in it has a path to `main`.

    codex/inventory-contract-alignment   PR #91  -> codex/fdu-opened-root-e2e-spike
    codex/fdu-opened-root-e2e-spike      NO PR   -> (nothing)
    codex/fdu-backend-alignment-research PR #74  -> main

Ancestry verified with `git merge-base --is-ancestor`: the stack is real, each layer does
build on the next. The problem is only that #91 targets a branch nobody is merging.

All three layers are **71 commits behind `origin/main`**:

    codex/inventory-contract-alignment    behind 71, ahead 28
    codex/fdu-opened-root-e2e-spike       behind 71, ahead 25
    codex/fdu-backend-alignment-research  behind 71, ahead 19

## What has to happen

Some order of: land #74, open a PR for the e2e-spike layer or fold its six commits into
one of its neighbours, retarget #91, and bring all of it current with `main`.

The 71-commit gap is the part with unknown cost. It is not yet known whether merging
`main` conflicts, and that has to be measured before any of this is scheduled rather than
assumed to be clean.

Nothing here is a code defect. It is entirely a landing-path problem, and it blocks the
end-to-end goal regardless of how correct the code on those branches is.
