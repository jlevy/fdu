---
type: is
id: is-01m4kjcscwzcrtw7gymgkxeg2k
title: Re-establish the Linux peer standing before the 0.5.0 README quotes it
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T18:49:50.236Z
updated_at: 2026-10-12T00:24:33.215Z
started_at: 2026-10-11T21:59:09.103Z
closed_at: 2026-10-12T00:24:33.213Z
close_reason: "Merged in PR #197 at 33a3a890: current Linux peer claims are qualified in README, performance measurements, and 0.5.0 release notes; no unsupported lead is claimed. Independent reviews, two full make check gates on the same source tree, and 20 PR CI checks passed."
resolution: null
duplicate_of: null
---
exp-217's tool cells (2026-10-10, 20 pairs, after a container restart, guest 6.18.44-fc-v114): 0.5.0's default command shows no resolved lead over pdu --max-depth 2 or diskus on node-modules-dense or linux-v6.12, and 0.4.0 is level with them too; all leads hold on linux-balanced-1m. exp-202's published leads (+1.8% and +1.9% lower bounds over pdu --max-depth 2) are not reproduced on this guest build. Before the release PR updates the README speed table and docs/performance-measurements.md: rerun the tool cells on a quiet host with more pairs, ideally the exp-202 guest build, and quote what holds.
