---
type: is
id: is-01m3q0d7kzfjb9spjp6m2va77x
title: Read gzipped run artifacts in the evidence tooling and compress this round's run.json files
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T16:36:46.335Z
updated_at: 2026-09-29T16:36:52.582Z
started_at: 2026-09-29T16:36:52.581Z
---
The PR diff for the 2026-09-29 Linux round is ~302K lines, 284K of them raw run.json artifacts. Teach the evidence tooling (validate, record, ledger, timeline, report) to read .json.gz via one helper; gzip exp-175..186 and exp-192..195 primary artifacts deterministically (mtime 0, no name); update run_artifact paths; add .gitattributes and tests; document the rule in performance-loop.md. gzip over zstd: stdlib in Python 3.12, no new dependency. Delegated to an Opus subagent on branch perf/evidence-gz.
