---
type: is
id: is-01m3q0d7kzfjb9spjp6m2va77x
title: Read gzipped run artifacts in the evidence tooling and compress this round's run.json files
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T16:36:46.335Z
updated_at: 2026-09-29T16:55:48.088Z
started_at: 2026-09-29T16:36:52.581Z
closed_at: 2026-09-29T16:55:48.088Z
close_reason: "Done in 1004e0fb on perf/evidence-gz: ledger.load reads .json.gz via stdlib gzip; record.py --run accepts it; 15 primary artifacts gzipped deterministically (8.1 MB -> 0.34 MB); 16 run_artifact paths updated; .gitattributes; 5 tests incl. every committed run_artifact exists and loads. PR diff 301,888 -> 17,925 added lines. Merge into the PR branch pending."
resolution: null
duplicate_of: null
---
The PR diff for the 2026-09-29 Linux round is ~302K lines, 284K of them raw run.json artifacts. Teach the evidence tooling (validate, record, ledger, timeline, report) to read .json.gz via one helper; gzip exp-175..186 and exp-192..195 primary artifacts deterministically (mtime 0, no name); update run_artifact paths; add .gitattributes and tests; document the rule in performance-loop.md. gzip over zstd: stdlib in Python 3.12, no new dependency. Delegated to an Opus subagent on branch perf/evidence-gz.
