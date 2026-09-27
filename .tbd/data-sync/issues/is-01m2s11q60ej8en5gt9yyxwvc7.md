---
type: is
id: is-01m2s11q60ej8en5gt9yyxwvc7
title: Watch --interval rejects 200ms and 0.2s
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-18-fdu-first-release-verification.md
labels: []
dependencies: []
parent_id: is-01m2s0tq4ppsygrs129nw1m86n
created_at: 2026-09-18T01:10:44.672Z
updated_at: 2026-09-27T08:15:13.058Z
closed_at: 2026-09-18T03:07:28.157Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
A stranger trying a snappier text watch writes --interval 200ms or 0.2s. Both are usage errors. 200ms: unknown age unit ms; 0.2s: fractional ages are not supported (example is 1h30m). The default is 2s; JSONL change records still stream without a sub-second interval. Not a publish blocker. Found in the 2026-09-18 new-user simulation (fdu-bnp9).
