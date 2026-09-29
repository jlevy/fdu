---
type: is
id: is-01m3q1w6yqkp9ec5nk7gdrwarh
title: Revise the README and user docs against the current code after the Linux round
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
created_at: 2026-09-29T17:02:25.751Z
updated_at: 2026-09-29T17:03:18.327Z
started_at: 2026-09-29T17:03:18.327Z
---
Stacked layer on #161 with the comparison matrix (fdu-dbn9), branch claude/readme-comparison-matrix. README's Speed bullet and section are stale for Linux (they say the tree view and .gitignore handling are still slower; the final head is level with pdu's default and diskus on real trees, and the default tree is 39% faster than 0.2.1 on linux-v6.12). Revise README.md, docs/README.md, docs/usage.md, docs/machine-output.md, crates/fdu/README.md, crates/fdu-core/README.md, crates/fdu-py/README.md and the bundled SKILL.md for accuracy against the current binary and engine (examples, flags, outputs, performance claims), and check fdu-engine-architecture.md and platform-tuning.md describe the transient tree tier and the Linux native reader. Every claim cites a current measurement; no answer or CLI change is implied by the round.
