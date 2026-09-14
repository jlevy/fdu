---
type: is
id: is-01m2h2wwwbp06dx4gs4q7qbmzp
title: Require a rejected experiment's candidate diff to stay reachable (patch or tag)
kind: task
status: open
priority: 3
version: 1
labels:
  - perf
  - stack-followup
dependencies: []
parent_id: is-01m2h2t7k65srszyhv02tag8ba
created_at: 2026-09-14T23:09:08.359Z
updated_at: 2026-09-14T23:09:08.359Z
---
PR #58, P3. The exp-104 `method` block pins the candidate binary (sha256 `146fde0c...`, 2,788,864 bytes) and describes the 66-line change in prose, but the diff itself is not recoverable: no patch, tag, or branch holds it. A search of every ref on origin, all tags, and the stash list found no candidate code (`impl Hash for PathKey` occurs only in the artifact's prose).

The runbook (`docs/project/guides/performance-loop-runbook.md`, DECIDE and COMMIT: "the code is reverted first") does not require a rejected candidate to be preserved, so this is a protocol gap, not a violation. It matters because a rejected record asks later work to be screened against it, and a reader cannot rebuild the measured candidate.

Ask: the protocol should require a rejected (or never-committed) candidate to leave its diff reachable, either as a patch under `docs/project/experiments/evidence/exp-NNN/` (the directory exp-102 established) named from the artifact body, or as a pushed tag, and `make perf-record` could take a `--candidate-patch` path and copy it there. Related: `fdu-c4jr` asks the same for the raw run JSON.
