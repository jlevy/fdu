---
type: is
id: is-01m0ptezmtmkn04mh1f1rwgdxb
title: "Loop job: what planes and groups cost on the reducer path"
kind: task
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:05:46.778Z
updated_at: 2026-08-25T06:30:29.043Z
---
Planes (fdu-mvt3) and groups (fdu-b2vy) each add maintained per-directory roll-up state, so together they multiply the ancestor-merge path rather than adding to it once. That is the path exp-064's H94 just made cheap (merge_ancestors 43.73% -> 14.07% of profile) and the one campaign 2 plans to delete rather than tune, in the fdu-cq7t follow-on it names the content-tier instance of H86 (key roll-ups by EntryId, one bottom-up pass). Measure the per-plane/per-group cost against that structural shape rather than against today's ancestor walk, on a dense real subject — exp-065 established that a sparse generated corpus flatters exactly this class of change. Coordinate with campaign 2 Phase C; these features supply the consumer requirement that makes the multiplication real.

## Notes

NOT RUN, and deliberately not attempted. Three preconditions fail, two of them on work
rather than on scheduling, so this is not merely waiting for a quiet machine.

1. The union is incomplete. The scope note says the reducer carries planes, browsing
   groups, composed subtree provenance and non-directory leaf counts "together or not at
   all, so measure the union rather than planes-times-groups alone". Planes (fdu-pxfz)
   and groups (fdu-b2vy) are in; composed subtree provenance is not -- fdu-fka6 and
   fdu-b1ts are both open. Measuring three of four would produce exactly the number the
   scope note warns against: a cost acceptable separately that is wrong in combination.

2. No admissible subject exists on this class of host. The nominated manifest is
   docs/project/reports/nominated-subjects-darwin-arm64.json -- Darwin, arm64, bare-metal
   -- and its subjects say so themselves: "Shape depends on which crates those lockfiles
   pull, so it is not a recipe another machine can follow." The dense real subject of 50k+
   entries this bead requires has to be nominated on the host that runs it.

3. exp-065 already ruled out the substitute. A sparse generated corpus flatters exactly
   this class of change, so materialising a synthetic 50k-entry tree instead would not be
   a weaker version of this measurement -- it would be the wrong one, and the ledger
   already records why.

The host constraint is the ordinary one on top: AGENTS.md is explicit that a timing gate
on a shared runner measures the runner, and a cloud container is that runner.

WHAT TO DO WHEN IT RUNS. fdu-cq7t is closed, so H86's replacement shape exists and the
"measure against the structural shape rather than today's ancestor walk" instruction is
now actionable rather than hypothetical. The sequence is: land fdu-fka6/fdu-b1ts, nominate
a dense subject on the measuring host with `make perf-subjects`, then `make perf-compare`
paired and interleaved with the four reducers on and off, `make perf-record`, and
republish with `make perf-ledger` + `make perf-report`.

One measurable claim this work already makes and did not verify: every added reducer is
opt-in and costs nothing when off. `count_dir_into_planes` and the plane arms iterate
`tag_rules.promoted()`, which is empty by default, so the default path takes one slice
deref per directory and no allocation. That is an argument from the code, not a
measurement, and it is the first thing the loop should confirm.
