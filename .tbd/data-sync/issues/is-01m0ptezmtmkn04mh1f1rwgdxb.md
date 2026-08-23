---
type: is
id: is-01m0ptezmtmkn04mh1f1rwgdxb
title: "Loop job: what planes and groups cost on the reducer path"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:05:46.778Z
updated_at: 2026-08-23T09:46:51.412Z
---
Planes (fdu-mvt3) and groups (fdu-b2vy) each add maintained per-directory roll-up state, so together they multiply the ancestor-merge path rather than adding to it once. That is the path exp-064's H94 just made cheap (merge_ancestors 43.73% -> 14.07% of profile) and the one campaign 2 plans to delete rather than tune, in the fdu-cq7t follow-on it names the content-tier instance of H86 (key roll-ups by EntryId, one bottom-up pass). Measure the per-plane/per-group cost against that structural shape rather than against today's ancestor walk, on a dense real subject — exp-065 established that a sparse generated corpus flatters exactly this class of change. Coordinate with campaign 2 Phase C; these features supply the consumer requirement that makes the multiplication real.

## Notes

2026-08-23: PR #45 (merged) built the instrument this bead needs. 'make perf-subjects' nominates a host's real trees by size and density and writes a redacted committable document; 'make perf-subjects-check' reports drift. A subject may decide an accept when it is dense and at least 50,000 entries, and a set may carry a ranking claim when its deciding subjects span three of four characters. So the dense-real-subject requirement is now a rule the harness enforces rather than a judgement call, and the generated corpus is explicitly disqualified there for the reason exp-065 found (depth-inflated and 22.6x sparse, which flatters exactly this class of change).
