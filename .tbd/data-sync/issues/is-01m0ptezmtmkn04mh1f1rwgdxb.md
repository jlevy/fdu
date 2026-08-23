---
type: is
id: is-01m0ptezmtmkn04mh1f1rwgdxb
title: "Loop job: what planes and groups cost on the reducer path"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:05:46.778Z
updated_at: 2026-08-23T20:34:46.186Z
---
Planes (fdu-mvt3) and groups (fdu-b2vy) each add maintained per-directory roll-up state, so together they multiply the ancestor-merge path rather than adding to it once. That is the path exp-064's H94 just made cheap (merge_ancestors 43.73% -> 14.07% of profile) and the one campaign 2 plans to delete rather than tune, in the fdu-cq7t follow-on it names the content-tier instance of H86 (key roll-ups by EntryId, one bottom-up pass). Measure the per-plane/per-group cost against that structural shape rather than against today's ancestor walk, on a dense real subject — exp-065 established that a sparse generated corpus flatters exactly this class of change. Coordinate with campaign 2 Phase C; these features supply the consumer requirement that makes the multiplication real.

## Notes

SCOPE EXTENDS to a four-way union. The reducer will carry planes, browsing groups, composed subtree provenance (fdu-fka6/fdu-b1ts), and non-directory leaf counts together or not at all, so measure the union rather than planes-times-groups alone — a cost acceptable for each separately can be wrong in combination. Still measured against H86's replacement shape rather than today's ancestor walk, on a dense real subject of 50k+ entries from make perf-subjects (exp-065: sparse generated corpora flatter this class).
